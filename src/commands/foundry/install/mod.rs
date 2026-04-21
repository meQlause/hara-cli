pub mod shell;

use std::process::{Command, Stdio};

/// Orchestrates the installation of Foundry.
pub fn run() -> Result<(), String> {
    tracing::info!("Installing Foundry for HARA development (Windows Git Bash)...");

    tracing::info!("Downloading foundryup installer...");

    let install_cmd = "curl -fsSL https://foundry.paradigm.xyz | bash";
    let shell_bin = shell::which_shell();

    let status = Command::new(&shell_bin)
        .args(["-c", install_cmd])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to run installer (requires bash + curl): {e}"))?;

    if !status.success() {
        let fpath = shell::foundryup_path();
        if fpath.exists() && fpath.to_string_lossy() != "foundryup.exe" {
            tracing::info!("Foundry installer returned non-zero, but foundryup.exe was found.");
            tracing::info!("Proceeding with installation...");
        } else {
            return Err(
                "Foundry installer failed.\n  Make sure curl and Git Bash are available."
                    .to_string(),
            );
        }
    }

    tracing::info!("foundryup downloaded.");
    tracing::info!("Running foundryup to install forge/cast/anvil...");

    let foundryup = shell::foundryup_path();
    let unix_path = foundryup.to_string_lossy().replace('\\', "/");

    let status = Command::new(&shell_bin)
        .args(["-c", &unix_path])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(_) => {
            tracing::info!("foundryup returned non-zero.");
            tracing::info!("Try restarting your terminal and running: foundryup");
            return Ok(());
        }
        Err(e) => {
            tracing::info!("Could not run foundryup automatically ({e}).");
            tracing::info!("Try restarting your terminal and run: foundryup");
            return Ok(());
        }
    }

    tracing::info!("Foundry installed successfully!");
    tracing::info!("Tools installed:");
    tracing::info!("  forge - smart contract build tool");
    tracing::info!("  cast  - CLI for EVM interactions");
    tracing::info!("  anvil - local Ethereum devnet");
    tracing::info!("Next step: cd into your project folder and run: hara init");
    Ok(())
}
