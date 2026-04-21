pub mod files;
pub mod git;

use std::fs;
use std::path::Path;
use crate::utils::forge::forge;

/// Orchestrates the initialization of a HARA project.
pub fn run() -> Result<(), String> {
    tracing::info!("Initialising HARA Foundry project...");

    if Path::new("foundry.toml").exists() {
        tracing::info!("Skipping forge init (foundry.toml already exists)");
    } else {
        tracing::info!("Running forge init...");
        let result = forge(&["init", "--force", "--no-git"]);
        if result.is_err() {
            forge(&["init", "--force"])?;
        }
    }

    git::init_repo()?;

    let deps = [
        ("OpenZeppelin Contracts v5.0.1", "openzeppelin/openzeppelin-contracts@v5.0.1"),
        ("OpenZeppelin Upgradeable Contracts v5.0.1", "openzeppelin/openzeppelin-contracts-upgradeable@v5.0.1"),
        ("Forge Standard Library", "foundry-rs/forge-std"),
    ];

    for (name, target) in deps {
        tracing::info!("Installing {}...", name);
        if let Err(e) = forge(&["install", target]) {
            tracing::info!("Note: {} (skipping...)", e);
        }
    }

    tracing::info!("Writing foundry.toml (HARA standard)...");
    fs::write("foundry.toml", files::FOUNDRY_TOML)
        .map_err(|e| format!("Failed to write foundry.toml: {e}"))?;
    tracing::info!("foundry.toml written");

    tracing::info!("Writing remappings.txt...");
    fs::write("remappings.txt", files::REMAPPINGS)
        .map_err(|e| format!("Failed to write remappings.txt: {e}"))?;
    tracing::info!("remappings.txt written");

    if !Path::new(".env").exists() {
        tracing::info!("Writing .env...");
        fs::write(".env", files::DOT_ENV).map_err(|e| format!("Failed to write .env: {e}"))?;
        tracing::info!(".env written");
    } else {
        tracing::info!("Skipping .env (already exists — not overwritten to protect secrets)");
    }

    tracing::info!("Writing .env.example...");
    fs::write(".env.example", files::DOT_ENV_EXAMPLE)
        .map_err(|e| format!("Failed to write .env.example: {e}"))?;
    tracing::info!(".env.example written");

    if !Path::new(".gitignore").exists() {
        tracing::info!("Writing .gitignore...");
        fs::write(".gitignore", files::GITIGNORE)
            .map_err(|e| format!("Failed to write .gitignore: {e}"))?;
        tracing::info!(".gitignore written");
    } else {
        tracing::info!("Skipping .gitignore (already exists)");
        files::append_gitignore_entry(".env")?;
    }

    tracing::info!("Running forge build to verify configuration...");
    forge(&["build"])?;

    tracing::info!("HARA project ready!");
    tracing::info!("Next steps:");
    tracing::info!("  1. Fill in your PRIVATE_KEY in .env");
    tracing::info!("  2. Run hara uc <ContractName> to scaffold your first contract");
    tracing::info!("  3. Run forge test to run the generated test suite");
    
    Ok(())
}
