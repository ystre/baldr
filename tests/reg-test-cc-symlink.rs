use assert_cmd::prelude::*;
use predicates::prelude::*;

use std::fs;
use std::path::Path;
use std::process::Command;

type AnyError = Result<(), Box<dyn std::error::Error>>;

const TARGET: &str = "tests/cpp/compile_commands.json";

fn command() -> Command {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!()).unwrap();

    cmd.args([
        "--project", "./tests/cpp",
        "--target", "test",
        "--delete",
        "--no-confirm",
    ]);

    cmd
}

#[test]
fn compile_commands_symlink() -> AnyError {
    command().assert().success().stderr(
        predicate::str::contains("`compile_commands.json` symlink already exists and is valid.").or(
        predicate::str::contains("Creating `compile_commands.json` symlink...")
    ));

    assert!(fs::symlink_metadata(Path::new(TARGET))?.is_symlink());
    Ok(())
}

#[test]
fn compile_commands_symlink_broken() -> AnyError {
    let target = Path::new(TARGET);

    let _ = std::fs::remove_file(target);
    std::os::unix::fs::symlink("/non-existent", target).unwrap();

    command().assert().success().stderr(predicate::str::contains("Broken `compile_commands.json` symlink is removed."));
    assert!(fs::symlink_metadata(target)?.is_symlink());

    std::fs::remove_file(target).unwrap();
    Ok(())
}
