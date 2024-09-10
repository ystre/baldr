use clap;

#[derive(clap::Args)]
pub struct ExeArgs {
    /// Run under debugger
    #[arg(long)]
    pub debug: bool,

    /// Arguments to be forwarded to the executable
    #[arg(last = true)]
    pub args: Vec<String>,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    Build{},
    Run(ExeArgs),
}

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

    /// Number of parallel build jobs
    #[arg(short, long, default_value_t = 1)]
    pub jobs: u8,

    /// Arguments to forward to CMake
    #[arg(short = 'D', long = "cmake-define")]
    pub cmake_args: Vec<String>,

    /// Overriding configuration file
    #[arg(long)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}
