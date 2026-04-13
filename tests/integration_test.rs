use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_uc_command_no_foundry_toml() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("hara").unwrap();
    cmd.current_dir(dir.path())
        .arg("uc")
        .arg("MyContract");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("'foundry.toml' not found"));
}

#[test]
fn test_uc_command_success() {
    let dir = tempdir().unwrap();
    // Create foundry.toml to satisfy the guard
    fs::write(dir.path().join("foundry.toml"), "").unwrap();

    let mut cmd = Command::cargo_bin("hara").unwrap();
    // Use an input that says "no" to the reset prompt
    cmd.current_dir(dir.path())
        .arg("uc")
        .arg("MyContract")
        .write_stdin("n\n");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("✅ Done!"));

    // Verify files were created
    assert!(dir.path().join("src/MyContract.sol").exists());
    assert!(dir.path().join("src/libraries/MyContractStorage.sol").exists());
}

#[test]
fn test_uc_command_reset() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("foundry.toml"), "").unwrap();
    
    // Create a directory that should be reset
    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("old_file.sol"), "").unwrap();

    let mut cmd = Command::cargo_bin("hara").unwrap();
    // Use an input that says "yes" to the reset prompt
    cmd.current_dir(dir.path())
        .arg("uc")
        .arg("MyContract")
        .write_stdin("y\n");

    cmd.assert()
        .success();

    // Verify old file is gone and new file is created
    assert!(!src_dir.join("old_file.sol").exists());
    assert!(dir.path().join("src/MyContract.sol").exists());
}
