pub mod cli;
pub mod cfg;
pub mod cmake;
pub mod utils;

use crate::utils::format_cmd;
use crate::cli::{Commands, ExeArgs};

use clap::Parser;
use config::Config;

use log::*;

use std::{
    io::{
        self,
        BufRead,
        Write
    },
    os::unix::fs,
    path::{
        Path,
        PathBuf
    },
    process::{
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
            "" => "".into(),
            _  => format!("-{}", Path::new(&self.compiler_path).file_name().unwrap().to_string_lossy().to_string()),
        };

        let dir = format!(
            "{}{}{}",
            self.build_type.to_lowercase(),
            compiler,
            match self.sanitizer {
                Some(san) => format!("-{}", san),
                None => "".into()
            }
        );

        PathBuf::from(self.project)
            .join("build")
            .join(dir)
    }
}

impl std::fmt::Display for BuildPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.to_path().display()))
    }
}

fn create_compile_cmd_symlink(src: &PathBuf, dst: &PathBuf) -> Result<(), io::Error> {
    let file = "compile_commands.json";
    if std::fs::exists(dst.join(file)).is_err() {
        return fs::symlink(src.join(file), dst.join(file));
    }
    Ok(())
}

fn delete_build_dir(build_dir: &PathBuf, confirm: bool) -> Result<bool, String> {
    if confirm {
        eprint!("Are you sure to remove `{}` (press 'y' to proceed): ", build_dir.to_string_lossy());
        io::stdout().lock().flush().unwrap();
        let answer = io::stdin().lock().lines().next().unwrap().unwrap();

        if answer != "y" {
            info!("Skipping clean build.");
            return Ok(false);
        }

    }
    else {
        info!("Non-interactive mode, skipping confirmation for deleting build directory.");
    }

    std::fs::remove_dir_all(&build_dir).map_err(|e| format!("Failed to delete build directory: {e}"))?;
    info!("Build directory deleted!");
    Ok(true)
}

fn build(args: &cli::Args, config: &Config, build_dir: &PathBuf, build_exists: bool) -> Result<(), String> {
    if !build_exists {
        cmake::configure(build_dir, args, config)?;
    } else if !args.no_configure {
        cmake::configure(build_dir, args, config)?;
    }

    cmake::build(build_dir, args)?;
    create_compile_cmd_symlink(build_dir, &args.project.clone().into())
        .map_err(|e| format!("Failed to create a symlink for `compile_commands.json`: {e}"))?;

    Ok(())
}

fn run(target: &String, build_dir: &PathBuf, config: &Config, args: &ExeArgs) -> Result<ExitStatus, String> {
    if target == "all" {
        return Err("Target must be specified".into());
    }

    let exes = utils::find_files(build_dir, |filename| { filename == *target });
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

            cmd.args(&args.args);
            let mut process = cmd.spawn().map_err(|e| format!("Failed to run the built executable: {e}"))?;
            let cmd_str = format_cmd(&cmd);
            Ok(process.wait().map_err(|e| format!("Command `{cmd_str}` did not start; {e}"))?)

        },
        0 => Err(format!("No executable found in `{}`", build_dir.display()).into()),
        _ => Err(format!("Multiple executables found in `{}`", build_dir.display()).into()),
    }
}

fn main() -> Result<(), String> {
    env_logger::builder()
        .format_timestamp_millis()
        .init();

    let args = cli::Args::parse();
    let config = cfg::read_config(&args.config).map_err(|e| e.to_string())?;

    let build_dir = BuildPath{
        project: args.project.as_str(),
        build_type: args.build_type.as_str(),
        compiler_path: &config.get_string("compiler.cxx").unwrap_or_default(),
        sanitizer: None
    }.to_path();

    info!("Using build directory: {}", build_dir.to_string_lossy());

    let mut build_exists = match std::fs::exists(&build_dir) {
        Ok(true) => {
            info!("Build directory already exists.");
            Ok(true)
        },
        Ok(false) => Ok(false),
        Err(x) => Err(format!("{}", x)),
    }?;

    if args.delete {
        if build_exists {
            build_exists = !delete_build_dir(&build_dir, !args.no_confirm)?;
        } else {
            warn!("Build directory does not exist, there is nothing to delete!");
        }

        if !build_exists {
            std::fs::create_dir_all(&build_dir).map_err(|e| format!("Failed to create build directory: {e}"))?;
            info!("Build directory has been created.");
        }
    }

    match &args.command {
        Commands::Build{} => {
            build(&args, &config, &build_dir, build_exists)?;
        }
        Commands::Run(exe_args) => {
            build(&args, &config, &build_dir, build_exists)?;
            run(&args.target, &build_dir, &config, exe_args).map_err(|e| format!("Failed to run executable: {e}"))?;
        }
    }

    Ok(())
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
