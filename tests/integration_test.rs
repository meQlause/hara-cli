use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_uc_command_no_foundry_toml() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("hara").unwrap();
    cmd.current_dir(dir.path())
        .args(["foundry", "contract", "uc", "MyContract"]);

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("'foundry.toml' not found"));
}

#[test]
fn test_uc_files_created() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("foundry.toml"), "").unwrap();

    // Provide "n" to the reset prompt; the forge build will fail (no deps in tmp),
    // but file scaffolding happens before that so we only check files exist.
    let mut cmd = Command::cargo_bin("hara").unwrap();
    cmd.current_dir(dir.path())
        .args(["foundry", "contract", "uc", "MyContract"])
        .write_stdin("n\n");

    // Allow failure (forge build will fail without deps) but check files were written.
    let _ = cmd.output();

    assert!(dir.path().join("src/MyContract.sol").exists(),
        "src/MyContract.sol should be scaffolded");
    assert!(dir.path().join("src/libraries/MyContractStorage.sol").exists(),
        "MyContractStorage.sol should be scaffolded");
}

#[test]
fn test_uc_command_reset_clears_old_files() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("foundry.toml"), "").unwrap();

    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("old_file.sol"), "").unwrap();

    let mut cmd = Command::cargo_bin("hara").unwrap();
    cmd.current_dir(dir.path())
        .args(["foundry", "contract", "uc", "MyContract"])
        .write_stdin("y\n");

    let _ = cmd.output();

    assert!(!src_dir.join("old_file.sol").exists(),
        "old file should have been deleted after reset");
    assert!(dir.path().join("src/MyContract.sol").exists(),
        "new contract file should exist after reset");
}
