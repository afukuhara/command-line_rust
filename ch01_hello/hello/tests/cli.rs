use std::process::Command;

use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};


#[test]
fn works() {
    let mut cmd = Command::new("ls");
    let res = cmd.output();
    assert!(res.is_ok());

    let mut cmd = Command::cargo_bin("hello").unwrap();
    cmd.assert().success().stdout("Hello, world!\n");
}

#[test]
fn true_ok() {
    let mut cmd = Command::cargo_bin("true").unwrap();
    cmd.assert().success();
}

#[test]
fn false_not_ok() {
    let mut cmd = Command::cargo_bin("false").unwrap();
    cmd.assert().failure();
    cmd.assert().code(1);
}

#[test]
fn abort_not_ok() {
    let mut cmd = Command::cargo_bin("false_abort").unwrap();
    cmd.assert().failure();
}
