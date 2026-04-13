use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn foundryup_path() -> PathBuf {
    let home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_default();

    let mut candidates: Vec<PathBuf> = vec![
        PathBuf::from(&home).join(".foundry").join("bin").join("foundryup"),
    ];

    // Windows: also check with .exe extension and the MSYS2/Git Bash home mapping
    #[cfg(windows)]
    candidates.push(PathBuf::from(&home).join(".foundry").join("bin").join("foundryup.exe"));

    for p in &candidates {
        if p.exists() {
            return p.clone();
        }
    }

    // Fall back to relying on PATH
    PathBuf::from("foundryup")
}

pub fn run() -> Result<(), String> {
    println!("Installing Foundry for HARA development...\n");

    println!("⬇️Downloading foundryup installer...");

    #[cfg(windows)]
    let install_cmd = "curl -fsSL https://foundry.paradigm.xyz | bash";
    #[cfg(not(windows))]
    let install_cmd = "curl -fsSL https://foundry.paradigm.xyz | bash";

    let shell = which_shell();
    let status = Command::new(&shell)
        .args(["-c", install_cmd])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to run installer (requires bash + curl): {e}"))?;

    if !status.success() {
        let fpath = foundryup_path();
        if fpath.exists() && fpath.to_string_lossy() != "foundryup" {
            println!("\n[Note] Foundry installer returned non-zero (likely shell detection failure), but foundryup was found.");
            println!("Proceeding with installation...");
        } else {
            return Err(
                "Foundry installer failed.\n  Make sure curl and bash (or Git Bash) are available."
                    .to_string(),
            );
        }
    }

    println!("\nfoundryup downloaded.");

    println!("\nRunning foundryup to install forge/cast/anvil...");

    let foundryup = foundryup_path();
    let shell = which_shell();

    let status = if cfg!(windows) {
        let unix_path = foundryup.to_string_lossy().replace('\\', "/");
        Command::new(&shell)
            .args(["-c", &unix_path])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
    } else {
        Command::new(&foundryup)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
    };

    match status {
        Ok(s) if s.success() => {}
        Ok(_) => {
            println!();
            println!("foundryup returned non-zero.");
            println!("    Try restarting your terminal and running: foundryup");
            return Ok(());
        }
        Err(e) => {
            println!();
            println!("Could not run foundryup automatically ({e}).");
            println!("    Restart your terminal and run: foundryup");
            return Ok(());
        }
    }

    println!();
    println!("Foundry installed successfully!");
    println!();
    println!("   forge  — smart contract build tool");
    println!("   cast   — CLI for EVM interactions");
    println!("   anvil  — local Ethereum devnet");
    println!();
    println!("Next step: cd into your project folder and run:");
    println!("   hara init");
    Ok(())
}

/// Returns "bash" on Linux, or the path to bash from Git Bash on Windows.
fn which_shell() -> String {
    // On Windows, Git Bash installs bash.exe in well-known locations.
    #[cfg(windows)]
    {
        let candidates = [
            r"C:\Program Files\Git\bin\bash.exe",
            r"C:\Program Files (x86)\Git\bin\bash.exe",
        ];
        for c in &candidates {
            if std::path::Path::new(c).exists() {
                return c.to_string();
            }
        }
    }
    "bash".to_string()
}
