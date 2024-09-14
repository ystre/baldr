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
    io,
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
        let compiler = Path::new(&self.compiler_path).file_name().unwrap();

        let dir = format!(
            "{}-{}{}",
            self.build_type.to_lowercase(),
            compiler.to_str().unwrap(),
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

fn build(args: &cli::Args, config: &Config, build_dir: &PathBuf) -> Result<(), String> {
    cmake::configure(build_dir, args, config)?;
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

                    let mut cmd = Command::new(debugger);
                    cmd.arg("--args");
                    cmd.arg(&exes[0]);
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
        compiler_path: &config.get_string("compiler").map_err(|e| e.to_string())?,
        sanitizer: None
    }.to_path();

    match std::fs::exists(&build_dir) {
        Ok(_) => info!("Build directory already exists at: {}", build_dir.to_string_lossy()),
        Err(_) => {
            std::fs::create_dir_all(&build_dir).expect("Failed to create directory");
            info!("Build directory created at: {}", build_dir.to_string_lossy());
        }
    }

    match &args.command {
        Commands::Build{} => {
            build(&args, &config, &build_dir)?;
        }
        Commands::Run(exe_args) => {
            build(&args, &config, &build_dir)?;
            run(&args.target, &build_dir, &config, exe_args).expect("Failed to run executable");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_dir() {
        assert_eq!(
            BuildPath{
                project: "project".into(),
                build_type: "Debug".into(),
                compiler_path: "gcc".into(),
                sanitizer: None
            }.to_path().to_string_lossy(),
            "project/build/debug-gcc"
        );

        assert_eq!(
            BuildPath{
                project: "project".into(),
                build_type: "Debug".into(),
                compiler_path: "gcc".into(),
                sanitizer: Some("asan".into())
            }.to_path().to_string_lossy(),
            "project/build/debug-gcc-asan"
        );
    }
}
