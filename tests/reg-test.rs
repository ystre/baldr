use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

type AnyError = Result<(), Box<dyn std::error::Error>>;

#[test]
fn build() -> AnyError {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!())?;

    cmd.args([
        "--project", "./tests/cpp",
        "--target", "test",
    ]);

    cmd.assert().success();

    Ok(())
}

#[test]
fn build_and_run() -> AnyError {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!())?;

    cmd.args([
        "--project", "./tests/cpp",
        "--target", "test",
        "--run"
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Arguments:"))
    ;

    Ok(())
}


#[test]
fn arguments() -> AnyError {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!())?;

    cmd.args([
        "--project", "./tests/cpp",
        "--target", "test",
        "--run",
        "--", "arg1", "arg2", "arg3"
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Arguments: arg1 arg2 arg3"))
    ;

    Ok(())
}

#[test]
fn cmake_definitions() -> AnyError {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!())?;

    cmd.args([
        "--project", "./tests/cpp",
        "--target", "test",
        "-DDEFINE1=v1",
        "--cmake-define", "DEFINE2=v2",
        "--run",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Defines: v1 v2"))
    ;

    Ok(())
}

#[test]
fn cmake_definitions_2() -> AnyError {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!())?;

    cmd.args([
        "--project", "./tests/cpp",
        "--target", "test",
        "-DDEFINE1=v17",
        "--cmake-define", "DEFINE2=v19",
        "--run",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Defines: v17 v19"))
    ;

    Ok(())
}

#[test]
fn cmake_definitions_3_no_configure() -> AnyError {
    let mut cmd1 = Command::cargo_bin(assert_cmd::crate_name!())?;

    cmd1.args([
        "--project", "./tests/cpp",
        "--target", "test",
        "-DDEFINE1=v17",
        "--run",
    ]);

    cmd1.assert().success();

    let mut cmd2 = Command::cargo_bin(assert_cmd::crate_name!())?;

    cmd2.args([
        "--project", "./tests/cpp",
        "--target", "test",
        "-DDEFINE1=v4",
        "--run",
        "--no-configure",
    ]);

    cmd2.assert()
        .success()
        .stdout(predicate::str::contains("Defines: v17"))
    ;

    Ok(())
}
