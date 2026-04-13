use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

const FOUNDRY_TOML: &str = r#"[profile.default]
evm_version = "paris"
src = "src"
out = "out"
libs = ["lib"]
solc_version = "0.8.29"
optimizer = true
optimizer_runs = 200

# Script permissions (recommended)
fs_permissions = [
    { access = "read", path = "." }
]

[rpc_endpoints]
hara = "${HARA_RPC_URL}"

[profile.ci]
fuzz.runs   = 512
gas_reports = ["*"]
"#;

const REMAPPINGS: &str = "@openzeppelin/=lib/openzeppelin-contracts/
@openzeppelin-upgradeable/=lib/openzeppelin-contracts-upgradeable/
@forge-std/=lib/forge-std/
forge-std/=lib/forge-std/src/
ds-test/=lib/forge-std/lib/ds-test/src/
";

const DOT_ENV: &str = "# Private key for deployment (without 0x prefix)
PRIVATE_KEY=your_private_key_here

# HARA RPC URL
HARA_RPC_URL=http://20.198.228.24:5625

# Optional: Gas configuration
GAS_LIMIT=30000000
GAS_PRICE=0
";

const DOT_ENV_EXAMPLE: &str = "# Private key for deployment (without 0x prefix)
PRIVATE_KEY=<your_private_key_without_0x>

# HARA RPC URL
HARA_RPC_URL=http://20.198.228.24:5625

# Optional: Gas configuration
GAS_LIMIT=30000000
GAS_PRICE=0
";

fn sh(cmd: &str) -> Result<(), String> {
    // Prepend ~/.foundry/bin so forge/cast/anvil are available even when the
    // shell hasn't been reloaded after `hara install`.
    let wrapped = format!(r#"export PATH="$HOME/.foundry/bin:$PATH"; {cmd}"#);
    let status = Command::new("bash")
        .args(["-c", &wrapped])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to run `{cmd}`: {e}"))?;

    if !status.success() {
        return Err(format!("`{cmd}` exited with non-zero status"));
    }
    Ok(())
}

pub fn run() -> Result<(), String> {
    println!("Initialising HARA Foundry project...\n");

    if Path::new("foundry.toml").exists() {
        println!("  ─  Skipping forge init (foundry.toml already exists)");
    } else {
        println!("Running forge init...");
        sh("forge init --no-git 2>/dev/null || forge init")?;
    }

    println!("\nInstalling OpenZeppelin Contracts v5.0.1...");
    sh("forge install openzeppelin/openzeppelin-contracts@v5.0.1 --no-commit")?;

    println!("\nInstalling OpenZeppelin Upgradeable Contracts v5.0.1...");
    sh("forge install openzeppelin/openzeppelin-contracts-upgradeable@v5.0.1 --no-commit")?;

    println!("\nInstalling Forge Standard Library...");
    sh("forge install foundry-rs/forge-std --no-commit")?;

    println!("\nWriting foundry.toml (HARA standard)...");
    fs::write("foundry.toml", FOUNDRY_TOML)
        .map_err(|e| format!("Failed to write foundry.toml: {e}"))?;
    println!("  ✔  foundry.toml");

    println!("\nWriting remappings.txt...");
    fs::write("remappings.txt", REMAPPINGS)
        .map_err(|e| format!("Failed to write remappings.txt: {e}"))?;
    println!("  ✔  remappings.txt");

    if !Path::new(".env").exists() {
        println!("\nWriting .env...");
        fs::write(".env", DOT_ENV)
            .map_err(|e| format!("Failed to write .env: {e}"))?;
        println!("  ✔  .env");
    } else {
        println!("\n  ─  Skipping .env (already exists — not overwritten to protect secrets)");
    }

    println!("\nWriting .env.example...");
    fs::write(".env.example", DOT_ENV_EXAMPLE)
        .map_err(|e| format!("Failed to write .env.example: {e}"))?;
    println!("  ✔  .env.example");

    append_gitignore_entry(".env")?;

    println!("\nRunning forge build to verify configuration...");
    sh("forge build")?;

    println!("\nHARA project ready!");
    println!();
    println!("  Next steps:");
    println!("    1. Fill in your PRIVATE_KEY in .env");
    println!("    2. Run  hara uc <ContractName>  to scaffold your first contract");
    println!("    3. Run  forge test               to run the generated test suite");
    Ok(())
}

fn append_gitignore_entry(entry: &str) -> Result<(), String> {
    let path = Path::new(".gitignore");
    let existing = if path.exists() {
        fs::read_to_string(path).map_err(|e| format!("Failed to read .gitignore: {e}"))?
    } else {
        String::new()
    };

    if !existing.lines().any(|l| l.trim() == entry) {
        let mut content = existing;
        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }
        content.push_str(entry);
        content.push('\n');
        fs::write(path, content)
            .map_err(|e| format!("Failed to update .gitignore: {e}"))?;
        println!("  ✔  Added '{entry}' to .gitignore");
    }
    Ok(())
}
