use std::path::Path;
use std::process::{Command, Stdio};

/// Initializes a git repository if one doesn't exist.
pub fn init_repo() -> Result<(), String> {
    if !Path::new(".git").exists() {
        tracing::info!("Initialising git repository (required for forge install)...");
        let status = Command::new("git")
            .args(["init"])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to run git init: {e}"))?;
        
        if !status.success() {
            return Err("git init failed. Make sure git is installed.".to_string());
        }
        tracing::info!("Git repository initialized");
    } else {
        tracing::info!("Git repository already exists, skipping git init");
    }
    Ok(())
}
