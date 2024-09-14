use walkdir::WalkDir;

use std::path::PathBuf;
use std::process::Command;

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

pub fn format_cmd(cmd: &Command) -> String {
    format!(
        "{} {}",
        cmd.get_program().to_string_lossy(),
        cmd.get_args().map(|x| x.to_string_lossy()).collect::<Vec<_>>().join(" ")
    )
}
