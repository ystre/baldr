use crate::utils::format_cmd;
use crate::cli;

use config::Config;
use log::*;

use std::process::{ExitStatus, Command};
use std::path::PathBuf;

pub fn configure(path: &PathBuf, args: &cli::Args, config: &Config) -> Result<ExitStatus, String> {
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

    for arg in &config.get_array("cmake.definitions").unwrap_or([].to_vec()) {
        cmd.arg(
            format!(
                "-D{}",
                arg.clone()
                    .into_string()
                    .map_err(|e| format!("CMake definition cannot be converted to string: {e}"))?
            )
        );
    }

    let cmd_str = format_cmd(&cmd);
    debug!("{}", cmd_str);
    let mut process = cmd.spawn().map_err(|e| format!("Spawning command `{cmd_str}` failed with `{e}`"))?;
    Ok(process.wait().map_err(|e| format!("Command `{cmd_str}` did not start; {e}"))?)
}

pub fn build(path: &PathBuf, args: &cli::Args) -> Result<ExitStatus, String> {
    let mut cmd = Command::new("cmake");
    cmd.args([
        "--build", &path.to_string_lossy(),
        "--target", &args.target,
        "--",
        "-j", args.jobs.to_string().as_str()
    ]);

    let cmd_str = format_cmd(&cmd);
    debug!("{}", cmd_str);
    let mut process = cmd.spawn().map_err(|e| format!("Spawning command `{cmd_str}` failed with `{e}`"))?;
    Ok(process.wait().map_err(|e| format!("Command `{cmd_str}` did not start; {e}"))?)
}
