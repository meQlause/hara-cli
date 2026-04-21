use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Locates the forge.exe binary, prioritizing the HARA-standard .foundry/bin path.
pub fn forge_bin() -> PathBuf {
    let home = env::var("USERPROFILE").unwrap_or_default();

    let candidates: Vec<PathBuf> = vec![
        PathBuf::from(&home).join(".foundry").join("bin").join("forge.exe"),
    ];

    for p in &candidates {
        if p.exists() {
            return p.clone();
        }
    }

    PathBuf::from("forge.exe")
}

/// Executes a forge command with the provided arguments, inheriting standard I/O.
pub fn forge(args: &[&str]) -> Result<(), String> {
    let bin = forge_bin();
    let display = format!("forge {}", args.join(" "));

    let status = Command::new(&bin)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| {
            format!(
                "Failed to launch `{display}`: {e}\n  \
                 Is Foundry installed? Run: hara foundry install"
            )
        })?;

    if !status.success() {
        return Err(format!("`{display}` exited with non-zero status"));
    }
    Ok(())
}
