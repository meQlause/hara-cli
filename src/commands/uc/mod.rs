mod templates;

use std::path::Path;
use crate::utils::{fs as ufs, names::ContractNames, prompt, forge::forge};

const DIRS: &[&str] = &["src", "src/libraries", "script", "test", ".github/workflows"];

pub fn run(raw_name: &str) -> Result<(), String> {
    if !Path::new("foundry.toml").exists() {
        return Err(
            "[ERROR] 'foundry.toml' not found. This command must be run from the root of a Foundry project.".to_string()
        );
    }

    let names = ContractNames::from_raw(raw_name);

    println!("Scaffolding upgradeable contract: {}\n", names.pascal);

    if prompt::ask_reset(DIRS) {
        ufs::reset_dirs(DIRS)?;
    }

    ufs::ensure_dirs(DIRS)?;

    ufs::write_file(
        &format!("src/{}.sol", names.pascal),
        &templates::contract::render(&names),
        true,
    )?;

    ufs::write_file(
        &format!("src/libraries/{}Storage.sol", names.pascal),
        &templates::storage::render(&names),
        true,
    )?;

    ufs::write_file(
        &format!("src/libraries/{}View.sol", names.pascal),
        &templates::view::render(&names),
        true,
    )?;

    ufs::write_file(
        &format!("script/Deploy{}.s.sol", names.pascal),
        &templates::deploy::render(&names),
        true,
    )?;

    ufs::write_file(
        &format!("test/{}.t.sol", names.pascal),
        &templates::test::render(&names),
        true,
    )?;

    ufs::write_file(
        "test/ContractLimits.t.sol",
        &templates::test_limits::render(&names),
        true,
    )?;

    ufs::write_file(
        ".github/workflows/contract-limits.yml",
        &templates::workflow_ci::render(&names),
        true,
    )?;

    ufs::write_file(
        &format!("src/{}V2.sol", names.pascal),
        &templates::contract_v2::render(&names),
        true,
    )?;

    ufs::write_file(
        &format!("src/libraries/{}V2Storage.sol", names.pascal),
        &templates::storage_v2::render(&names),
        true,
    )?;

    ufs::write_file(
        &format!("script/Upgrade{}.s.sol", names.pascal),
        &templates::deploy_upgrade::render(&names),
        true,
    )?;

    ufs::write_if_missing(
        "src/libraries/Structs.sol",
        &templates::structs::render(),
    )?;

    ufs::write_if_missing(
        "src/libraries/Errors.sol",
        &templates::errors::render(),
    )?;

    ufs::write_if_missing(
        "src/libraries/Events.sol",
        &templates::events::render(),
    )?;

    println!("\n🔨 Running forge build to verify configuration...");
    forge(&["build"])?;

    println!("\nDone!");
    Ok(())
}
