use clap::Parser;
use config::Config;

use baldr::{
    Args,
    build,
    configure,
    find_files,
    format_cmd,
    read_config,
    read_input,
};

use log::*;

use std::{
    io,
    fs,
    fmt,
    os::unix::fs::symlink,
    path::{
        self,
        Path,
        PathBuf
    },
    process::{
        self,
        Command,
        ExitStatus
    }
};

struct BuildPath<'a> {
    project: &'a str,
    build_type: &'a str,
    compiler_path: &'a str,
    sanitizer: Option<&'a str>
}

impl<'a> BuildPath<'a> {

    /// Create a build path with the build dir containing the following information.
    ///
    /// - build type in lowercase, e.g. `debug` or `release`
    /// - compiler name - source: CC and CXX
    /// - compiler version (if not the default is in use) - source: CC and CXX
    /// - sanitizers (if used)
    ///
    fn to_path(&self) -> PathBuf {
        let compiler = match self.compiler_path {
            "" => String::new(),
            _  => format!("-{}", Path::new(&self.compiler_path).file_name().expect("Invalid compiler path").to_string_lossy()),
        };

        let dir = format!(
            "{}{}{}",
            self.build_type.to_lowercase(),
            compiler,
            match self.sanitizer {
                Some(san) => format!("-{san}"),
                None => String::new()
            }
        );

        PathBuf::from(self.project)
            .join("build")
            .join(dir)
    }
}

impl fmt::Display for BuildPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.to_path().display()))
    }
}

fn create_compile_cmd_symlink(src: &Path, dst: &Path) -> Result<(), io::Error> {
    let file = "compile_commands.json";
    let src = path::absolute(src.join(file))?;
    let dst = dst.join(file);

    match fs::exists(&dst) {
        Ok(true) => {
            debug!("`compile_commands.json` symlink already exists and is valid.");
            Ok(())
        }
        Ok(false) => {
            match fs::remove_file(&dst) {
                Ok(()) => { debug!("Broken `compile_commands.json` symlink is removed."); },
                Err(e) => { debug!("`compile_commands.json` symlink cannot be removed: {e}"); },
            };

            debug!("Creating `compile_commands.json` symlink...");
            symlink(src, &dst)
        }
        Err(e) => Err(e),
    }
}

fn delete_build_dir(build_dir: &Path, confirm: bool) -> Result<bool, String> {
    if confirm {
        eprint!("Are you sure to remove `{}` (press 'y' to proceed): ", build_dir.to_string_lossy());

        if read_input() != "y" {
            info!("Skipping clean build.");
            return Ok(false);
        }

    }
    else {
        info!("Non-interactive mode, skipping confirmation for deleting build directory.");
    }

    fs::remove_dir_all(build_dir).map_err(|e| format!("Failed to delete build directory: {e}"))?;
    info!("Build directory deleted!");
    Ok(true)
}

fn run(target: &String, build_dir: &PathBuf, config: &Config, args: &Args) -> Result<ExitStatus, String> {
    if target == "all" {
        return Err("Target must be specified".into());
    }

    let exes = find_files(build_dir, |filename| { filename == *target });
    match exes.len() {
        1 => {
            let mut cmd = (|| -> Result<Command, String> {
                if args.debug {
                    let debugger = config.get::<String>("debugger")
                        .map_err(|e| format!("No debugger is configured: {e}"))?;

                    let mut cmd = Command::new(&debugger);
                    if debugger == "gdb" {
                        cmd.arg("--args");
                        cmd.arg(&exes[0]);
                    } else if debugger == "lldb" {
                        cmd.arg(&exes[0]);
                    }
                    Ok(cmd)
                } else {
                    Ok(Command::new(&exes[0]))
                }
            })()?;

            cmd.args(&args.exe_args);
            let mut process = cmd.spawn().map_err(|e| format!("Failed to run the built executable: {e}"))?;
            let cmd_str = format_cmd(&cmd);
            Ok(process.wait().map_err(|e| format!("Command `{cmd_str}` did not start; {e}"))?)

        },
        0 => Err(format!("No executable found in `{}`", build_dir.display())),
        _ => Err(format!("Multiple executables found in `{}`", build_dir.display())),
    }
}

fn entrypoint() -> Result<(), String> {
    let args = Args::parse();
    let config = read_config(&args.config).map_err(|e| e.to_string())?;

    let build_dir = BuildPath{
        project: args.project.as_str(),
        build_type: args.build_type.as_str(),
        compiler_path: &config.get_string("compiler.cxx").unwrap_or_default(),
        sanitizer: None
    }.to_path();

    info!("Using build directory: {}", build_dir.to_string_lossy());

    let mut build_exists = match fs::exists(&build_dir) {
        Ok(true) => {
            info!("Build directory already exists.");
            Ok(true)
        },
        Ok(false) => Ok(false),
        Err(x) => Err(format!("{x}")),
    }?;

    if args.delete {
        if build_exists {
            build_exists = !delete_build_dir(&build_dir, !args.no_confirm)?;
        } else {
            warn!("Build directory does not exist, there is nothing to delete!");
        }

        if !build_exists {
            fs::create_dir_all(&build_dir).map_err(|e| format!("Failed to create build directory: {e}"))?;
            info!("Build directory has been created.");
        }
    }

    if !build_exists || !args.no_configure {
        configure(build_dir.as_path(), &args, &config)?;
    }

    if !build(build_dir.as_path(), &args)?.success() {
        return Err("Build failed".into());
    }

    create_compile_cmd_symlink(build_dir.as_path(), Path::new(&args.project))
        .map_err(|e| format!("Failed to create a symlink for `compile_commands.json`: {e}"))?;

    if args.run {
        run(&args.target, &build_dir, &config, &args)?;
    }

    Ok(())
}

fn main() {
    env_logger::builder()
        .format_timestamp_millis()
        .init();

    match entrypoint() {
        Ok(()) => {},
        Err(e) => {
            log::error!("Fatal error encountered: {e}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_dir_default() {
        assert_eq!(
            BuildPath{
                project: "project".into(),
                build_type: "Debug".into(),
                compiler_path: "".into(),
                sanitizer: None
            }.to_path().to_string_lossy(),
            "project/build/debug"
        );
    }

    #[test]
    fn build_dir_sanitizer() {
        assert_eq!(
            BuildPath{
                project: "project".into(),
                build_type: "Debug".into(),
                compiler_path: "".into(),
                sanitizer: Some("asan")
            }.to_path().to_string_lossy(),
            "project/build/debug-asan"
        );
    }

    #[test]
    fn build_dir_compiler() {
        assert_eq!(
            BuildPath{
                project: "project".into(),
                build_type: "Debug".into(),
                compiler_path: "gcc".into(),
                sanitizer: None
            }.to_path().to_string_lossy(),
            "project/build/debug-gcc"
        );
    }

    #[test]
    fn build_dir_both() {
        assert_eq!(
            BuildPath{
                project: "project".into(),
                build_type: "Debug".into(),
                compiler_path: "gcc".into(),
                sanitizer: Some("asan")
            }.to_path().to_string_lossy(),
            "project/build/debug-gcc-asan"
        );
    }
}
