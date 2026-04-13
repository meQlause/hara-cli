use std::process::{Command, Stdio};

pub fn run() -> Result<(), String> {
    println!("Installing Foundry for HARA development...\n");

    println!("⬇Downloading foundryup installer...");
    let status = Command::new("bash")
        .args(["-c", "curl -fsSL https://foundry.paradigm.xyz | bash"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to run installer: {e}"))?;

    if !status.success() {
        return Err("Foundry installer failed. Make sure curl and bash are available.".to_string());
    }

    println!();
    println!("foundryup installed.");

    println!("\nRunning foundryup to install forge/cast/anvil...");
    let status = Command::new("bash")
        .args([
            "-c",
            r#"export PATH="$HOME/.foundry/bin:$PATH"; source ~/.bashrc 2>/dev/null || true; source ~/.bash_profile 2>/dev/null || true; foundryup"#,
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to run foundryup: {e}"))?;

    if !status.success() {
        println!();
        println!("foundryup returned non-zero. If foundryup was just installed you may need to");
        println!("    restart your terminal and run: foundryup");
        return Ok(()); 
    }

    println!();
    println!("Foundry installed successfully!");
    println!();
    println!("  forge  — smart contract build tool");
    println!("  cast   — CLI for EVM interactions");
    println!("  anvil  — local Ethereum devnet");
    println!();
    println!("Next step: cd into your project folder and run:");
    println!("   hara init");
    Ok(())
}
