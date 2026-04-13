use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn forge_bin() -> PathBuf {
    let home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_default();

    let mut candidates: Vec<PathBuf> = vec![
        PathBuf::from(&home).join(".foundry").join("bin").join("forge"),
    ];

    #[cfg(windows)]
    candidates.push(
        PathBuf::from(&home)
            .join(".foundry")
            .join("bin")
            .join("forge.exe"),
    );

    for p in &candidates {
        if p.exists() {
            return p.clone();
        }
    }

    // Fall back to PATH
    PathBuf::from(if cfg!(windows) { "forge.exe" } else { "forge" })
}

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
                 Is Foundry installed? Run: hara install"
            )
        })?;

    if !status.success() {
        return Err(format!("`{display}` exited with non-zero status"));
    }
    Ok(())
}
