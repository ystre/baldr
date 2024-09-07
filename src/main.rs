use clap::Parser;
use log::*;

use std::{
    env,
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
    fn to_path(self) -> PathBuf {
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

fn create_compile_cmd_symlink(src: &PathBuf, dst: &PathBuf) -> Result<(), io::Error> {
    let file = "compile_commands.json";
    fs::symlink(src.join(file), dst.join(file))
}

fn build(path: &PathBuf, build_type: &String) -> Result<ExitStatus, io::Error> {
    let mut cmd = Command::new("ech")
        .arg(format!("Path: {:?}, build: {}", path, build_type))
        // .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process!");

    Ok(cmd.wait()?)
    // let result = cmd.wait_with_output().unwrap();
    // info!("{}", String::from_utf8(result.stdout.as_slice().to_vec()).unwrap());
}

#[derive(Parser)]
struct Args {
    /// Project path to build
    #[arg(short, long)]
    project: String,

    /// Build type
    #[arg(short, long, default_value_t = String::from("Debug"))]
    build_type: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .format_timestamp_nanos()
        .init();

    let args = Args::parse();
    let compiler = (env::var("CC")?, env::var("CXX")?);

    let build_dir = BuildPath{
        project: args.project.as_str(),
        build_type: args.build_type.as_str(),
        compiler_path: compiler.1.as_str(),
        sanitizer: None
    }.to_path();

    std::fs::create_dir(&build_dir)?;
    info!("Build directory created at: {}", build_dir.to_string_lossy());

    create_compile_cmd_symlink(&build_dir, &args.project.into())?;

    build(&build_dir, &args.build_type)?;

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
