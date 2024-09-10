use clap;

#[derive(clap::Args)]
pub struct ExeArgs {
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
#[derive(clap::Parser)]
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

    #[command(subcommand)]
    pub command: Commands,
}
