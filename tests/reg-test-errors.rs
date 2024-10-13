use assert_cmd::prelude::*;
use predicates::prelude::*;

use std::process::Command;

type AnyError = Result<(), Box<dyn std::error::Error>>;

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

/// The argument passed to the built executable is the exit code
/// if it is the only argument.
#[test]
fn exit_codes() -> AnyError {
    let exit_code = "9";

    command()
        .args(["--run", "--", exit_code])
        .assert().failure()
        .stderr(predicate::str::contains(format!("Process has returned with exit code: {exit_code}")))
    ;

    Ok(())
}

#[test]
fn unsupported_debugger() -> AnyError {
    command()
        .args(["--debug", "--run"])
        .env("BALDR_DEBUGGER", "unknown")
        .assert().failure()
        .stderr(predicate::str::contains("Unsupported debugger: "))
    ;

    Ok(())
}
