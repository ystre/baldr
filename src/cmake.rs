use crate::cfg::get_cmake_definitions;
use crate::utils::format_cmd;
use crate::cli;

use config::Config;
use log::*;

use std::path::Path;
use std::process::{ExitStatus, Command};

/// Invoke CMake's configure command.
///
/// # Errors
///
/// Returns an error, if the process cannot be started.
pub fn configure(path: &Path, args: &cli::Args, config: &Config) -> Result<ExitStatus, String> {
    let mut cmd = Command::new("cmake");

    let cc: String = config.get("compiler.cc").unwrap_or_default();
    let cxx: String = config.get("compiler.cxx").unwrap_or_default();
    if !cc.is_empty() && !cxx.is_empty() {
        cmd.env("CC", cc);
        cmd.env("CXX", cxx);
    }

    cmd.args([
        "-S", args.project.as_str(),
        "-B", &path.to_string_lossy(),
        format!("-DCMAKE_BUILD_TYPE={}", args.build_type).as_str(),
        "-DCMAKE_EXPORT_COMPILE_COMMANDS=ON"
    ]);

    for arg in &args.cmake_args {
        cmd.arg(format!("-D{arg}"));
    }

    for arg in get_cmake_definitions(config) {
        cmd.arg(format!("-D{arg}"));
    }

    let cmd_str = format_cmd(&cmd);
    debug!("{}", cmd_str);
    let mut process = cmd.spawn().map_err(|e| format!("Spawning command `{cmd_str}` failed with `{e}`"))?;
    process.wait().map_err(|e| format!("Command `{cmd_str}` did not start; {e}"))
}

/// Invoke CMake's build command.
///
/// # Errors
///
/// Returns an error, if the process cannot be started.
pub fn build(path: &Path, args: &cli::Args) -> Result<ExitStatus, String> {
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
    process.wait().map_err(|e| format!("Command `{cmd_str}` did not start; {e}"))
}
