use crate::utils::debug_cmd;
use crate::cli;

use config::Config;

use std::io;
use std::process::{ExitStatus, Command};
use std::path::PathBuf;

pub fn configure(path: &PathBuf, args: &cli::Args, config: &Config) -> Result<ExitStatus, io::Error> {
    // TODO(feat): configure optional CLI arguments from config file
    let mut cmd = Command::new("cmake");
    cmd.args([
        "-S", args.project.as_str(),
        "-B", &path.to_string_lossy(),
        format!("-DCMAKE_BUILD_TYPE={}", args.build_type).as_str(),
        "-DCMAKE_EXPORT_COMPILE_COMMANDS=ON"
    ]);

    for arg in &args.cmake_args {
        cmd.arg(format!("-D{}", arg));
    }

    let mut process = cmd.spawn().expect("Failed to spawn CMake configuring");
    debug_cmd(&cmd);

    Ok(process.wait()?)
}

pub fn build(path: &PathBuf, args: &cli::Args, config: &Config) -> Result<ExitStatus, io::Error> {
    // TODO(feat): configure optional CLI arguments from config file
    let mut cmd = Command::new("cmake");
    cmd.args([
        "--build", &path.to_string_lossy(),
        "--target", &args.target,
        "--",
        "-j", args.jobs.to_string().as_str()
    ]);

    let mut process = cmd.spawn().expect("Failed to spawn CMake build");
    debug_cmd(&cmd);

    Ok(process.wait()?)
}
