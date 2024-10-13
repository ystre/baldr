use std::env;
use std::io::{self, BufRead, Write};
use std::path::{Path,PathBuf};
use std::process::{ExitStatus, Command};

use config::builder::DefaultState;
use config::{Config, ConfigBuilder};
use walkdir::WalkDir;

use log::*;

/// Baldur, a C++/CMake project builder.
///
/// Build, run and debug.
///
/// Additional configuration is done via config files. Lookup in order (last has the highest
/// priority):
/// * XDG_CONFIG_HOME
/// * HOME
/// * Current working directory
///
/// If multiple files found, they are merged. In case of keys defined in multiple places, the
/// highest priority will be kept.
///
/// The path can be overridden via `--config`, in which case it is the only file read, without
/// merging.
///
/// The name of the file is `baldr.yaml` for example (unless overridden). The extension is
/// automatically recognized. The followings are supported:
/// * TOML
/// * JSON
/// * YAML
/// * INI
/// * RON
/// * JSON5
#[derive(clap::Parser)]
#[command(verbatim_doc_comment)]
pub struct Args {
    /// Project path to build (containing the root CMakeLists.txt)
    #[arg(short, long)]
    pub project: String,

    /// Build type
    #[arg(short, long, default_value_t = String::from("Debug"))]
    pub build_type: String,

    /// Overriding build directory
    #[arg(long)]
    pub build_dir: Option<String>,

    /// CMake target to build
    #[arg(short, long, default_value_t = String::from("all"))]
    pub target: String,

    /// Clean build
    #[arg(short, long, default_value_t = false)]
    pub delete: bool,

    /// Skip confirmations (can be handy for scripting)
    #[arg(long, default_value_t = false)]
    pub no_confirm: bool,

    /// Skip CMake configure (only applicable if it is already configured)
    #[arg(long, default_value_t = false)]
    pub no_configure: bool,

    /// Number of parallel build jobs
    #[arg(short, long, default_value_t = 1)]
    pub jobs: u8,

    /// Arguments to forward to CMake
    #[arg(short = 'D', long = "cmake-define")]
    pub cmake_args: Vec<String>,

    /// Overriding configuration file
    #[arg(long)]
    pub config: Option<String>,

    /// Run the built binary
    #[arg(short, long, default_value_t=false)]
    pub run: bool,

    /// Run under debugger
    #[arg(long)]
    pub debug: bool,

    /// Arguments to be forwarded to the executable
    #[arg(last = true)]
    pub exe_args: Vec<String>,
}

fn read_one_config(var: &str, cfg: ConfigBuilder<DefaultState>) -> ConfigBuilder<DefaultState> {
    if let Ok(x) = env::var(var) {
        log::debug!("Looking for config in {var}.");

        let config_path = Path::new(&x).join("baldr");

        cfg.add_source(
            config::File::with_name(config_path.to_str().expect("Non UTF-8 string in path")
        ).required(false))
    }
    else {
        log::debug!("{var} is not defined.");
        cfg
    }
}

/// Read configuration from environment variables and files.
///
/// Files are looked in the following directories:
/// * XDG_CONFIG_HOME
/// * HOME
/// * Current working directory
///
/// # Errors
///
/// Returns an error if config files exist but cannot be read or the configuration is invalid.
pub fn read_config(config_override: &Option<String>) -> Result<Config, config::ConfigError> {
    let mut config = Config::builder();

    config = match config_override {
        Some(x) => {
            config.add_source(config::File::with_name(x.as_str()))
        },
        None => {
            config = read_one_config("XDG_CONFIG_HOME", config);
            config = read_one_config("HOME", config);
            config.add_source(config::File::with_name("./baldr").required(false))
        }
    };

    config
        .add_source(config::Environment::with_prefix("BALDR"))
        .build()
}

pub fn get_cc(cfg: &Config) -> Option<String> {
    match cfg.get_string("compiler.cc") {
        Ok(x) => Some(x),
        Err(_) => None,
    }
}

pub fn get_cxx(cfg: &Config) -> Option<String> {
    match cfg.get_string("compiler.cxx") {
        Ok(x) => Some(x),
        Err(_) => None,
    }
}

pub fn get_cmake_definitions(cfg: &Config) -> Vec<String> {
    match cfg.get_array("cmake.definitions") {
        Ok(x) =>
            x.iter()
                .filter_map(|x| x.clone().into_string().ok())
                .collect(),
        Err(_) => [].to_vec(),
    }
}

/// Invoke CMake's configure command.
///
/// # Errors
///
/// Returns an error, if the process cannot be started.
pub fn configure(path: &Path, args: &Args, config: &Config) -> Result<ExitStatus, String> {
    let mut cmd = Command::new("cmake");

    let cc: String = get_cc(config).unwrap_or_default();
    let cxx: String = get_cxx(config).unwrap_or_default();
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
    debug!("CMD: {}", cmd_str);
    let mut process = cmd.spawn().map_err(|e| format!("Spawning command `{cmd_str}` failed with `{e}`"))?;
    process.wait().map_err(|e| format!("Command `{cmd_str}` did not start; {e}"))
}

/// Invoke CMake's build command.
///
/// # Errors
///
/// Returns an error, if the process cannot be started.
pub fn build(path: &Path, args: &Args) -> Result<ExitStatus, String> {
    let mut cmd = Command::new("cmake");
    cmd.args([
        "--build", &path.to_string_lossy(),
        "--target", &args.target,
        "--",
        "-j", args.jobs.to_string().as_str()
    ]);

    let cmd_str = format_cmd(&cmd);
    debug!("CMD: {}", cmd_str);
    let mut process = cmd.spawn().map_err(|e| format!("Spawning command `{cmd_str}` failed with `{e}`"))?;
    process.wait().map_err(|e| format!("Command `{cmd_str}` did not start; {e}"))
}

/// Recursively searches for files in a directory and applies a callback to filter the results.
///
/// # Arguments
/// * `directory` - The root directory where the search starts.
/// * `callback` - A closure or function that takes a file name and returns a boolean value indicating whether the file should be included.
///
/// # Returns
/// A vector of file paths that satisfy the callback condition.
pub fn find_files<F>(directory: &PathBuf, callback: F) -> Vec<String>
where
    F: Fn(&str) -> bool,
{
    let mut found_files = Vec::new();

    for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                if callback(file_name) {
                    found_files.push(path.display().to_string());
                }
            }
        }
    }

    found_files
}

/// Format a command.
///
/// Useful for debugging purposes.
///
/// ```
/// use std::process::Command;
/// use baldr::format_cmd;
///
/// let mut cmd = Command::new("echo");
/// cmd.args(["hello", "there"]);
///
/// assert_eq!(format_cmd(&cmd), "echo hello there");
/// ```
pub fn format_cmd(cmd: &Command) -> String {
    format!(
        "{} {}",
        cmd.get_program().to_string_lossy(),
        cmd.get_args().map(|x| x.to_string_lossy()).collect::<Vec<_>>().join(" ")
    )
}

/// Read input from `stdin`.
///
/// # Panics
///
/// Will panic in case of IO error. Cannot be handled in any meaningful way.
#[allow(clippy::unwrap_used)]
pub fn read_input() -> String {
    io::stdout().lock().flush().unwrap();
    io::stdin().lock().lines().next().unwrap().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config::builder().add_source(
            config::File::with_name(
                Path::new("baldr")
                    .to_str()
                    .unwrap()
            )
        ).build().unwrap()
    }

    #[test]
    fn cfg_cc() {
        assert_eq!(get_cc(&config()), Some("gcc".into()));
    }

    #[test]
    fn cfg_cxx() {
        assert_eq!(get_cxx(&config()), Some("g++".into()));
    }

    #[test]
    fn cfg_cmake_definitions() {
        assert_eq!(
            get_cmake_definitions(&config()),
            vec![
                "CFG1=cfg1",
                "CFG2=cfg2",
            ]
        );
    }
}
