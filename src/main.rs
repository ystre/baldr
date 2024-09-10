pub mod cli;
pub mod cfg;
pub mod cmake;
pub mod utils;

use crate::utils::debug_cmd;
use crate::cli::{Commands, ExeArgs};

use clap::Parser;
use config::Config;

use log::*;

use std::{
    error::Error,
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

fn build(args: &cli::Args, config: &Config, build_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    cmake::configure(build_dir, args, config)?;
    cmake::build(build_dir, args, config)?;
    create_compile_cmd_symlink(build_dir, &args.project.clone().into())?;

    Ok(())
}

fn run(target: &String, build_dir: &PathBuf, config: &Config, args: &ExeArgs) -> Result<ExitStatus, Box<dyn Error>> {
    if target == "all" {
        return Err("Target must be specified".into());
    }

    let exes = utils::find_files(build_dir, |filename| { filename == *target });
    match exes.len() {
        1 => {
            let mut cmd = (|| {
                if args.debug {
                    let debugger = config.get::<String>("debugger")
                        .expect("No debugger is configured");                                       // TODO(err): proper and consistent error handling

                    let mut cmd = Command::new(debugger);
                    cmd.arg("--args");
                    cmd.arg(&exes[0]);
                    cmd
                } else {
                    Command::new(&exes[0])
                }
            })();

            cmd.args(&args.args);
            let mut process = cmd.spawn().expect("Failed to run the built executable");             // TODO(err): proper and consistent error handling
            debug_cmd(&cmd);
            Ok(process.wait()?)

        },
        0 => Err(format!("No executable found in `{}`", build_dir.display()).into()),
        _ => Err(format!("Multiple executables found in `{}`", build_dir.display()).into()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .format_timestamp_millis()
        .init();

    let args = cli::Args::parse();
    let config = cfg::read_config(&args.config).unwrap();

    let build_dir = BuildPath{
        project: args.project.as_str(),
        build_type: args.build_type.as_str(),
        // compiler_path: compiler.1.as_str(),
        compiler_path: "na",      // TODO(feat): configurable compilers
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
            build(&args, &config, &build_dir).expect("Build failed");
        }
        Commands::Run(exe_args) => {
            build(&args, &config, &build_dir).expect("Build failed");
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
