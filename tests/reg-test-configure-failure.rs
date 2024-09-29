use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

type AnyError = Result<(), Box<dyn std::error::Error>>;

#[test]
fn configure_failure() -> AnyError {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!())?;

    cmd.args([
        "--project", "./tests/cpp",
        "--target", "test",
        "-DCONFIGURE_FAILURE=1",
        "--delete",
        "--no-confirm",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Fatal error encountered: Configuring failed"))
    ;

    Ok(())
}
