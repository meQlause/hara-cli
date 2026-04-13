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

const GITIGNORE: &str = r#"# Foundry
out/
cache/
broadcast/
.env

# Node
node_modules/
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log

# Runtime data
pids
*.pid
*.seed
*.pid.lock

# Coverage directory used by tools like istanbul
coverage/
*.lcov

# nyc test coverage
.nyc_output

# Dependency directories
jspm_packages/

# Optional npm cache directory
.npm

# Optional eslint cache
.eslintcache

# Microbundle cache
.rpt2_cache/
.rts2_cache_cjs/
.rts2_cache_es/
.rts2_cache_umd/

# Optional REPL history
.node_repl_history

# Output of 'npm pack'
*.tgz

# Yarn Integrity file
.yarn-integrity

# dotenv environment variables file
.env.test
.env.production

lib
"#;

use crate::utils::forge::forge;


pub fn run() -> Result<(), String> {
    println!("Initialising HARA Foundry project...\n");

    if Path::new("foundry.toml").exists() {
        println!("  ─  Skipping forge init (foundry.toml already exists)");
    } else {
        println!("Running forge init...");
        let result = forge(&["init", "--force", "--no-git"]);
        if result.is_err() {
            forge(&["init", "--force"])?;
        }
    }

    if !Path::new(".git").exists() {
        println!("\nInitialising git repository (required for forge install)...");
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
        println!("  ✔  git init");
    } else {
        println!("\n  ─  Git repository already exists, skipping git init");
    }

    println!("\nInstalling OpenZeppelin Contracts v5.0.1...");
    if let Err(e) = forge(&["install", "openzeppelin/openzeppelin-contracts@v5.0.1"]) {
        println!("  ─  Note: {} (skipping...)", e);
    }

    println!("\nInstalling OpenZeppelin Upgradeable Contracts v5.0.1...");
    if let Err(e) = forge(&["install", "openzeppelin/openzeppelin-contracts-upgradeable@v5.0.1"]) {
        println!("  ─  Note: {} (skipping...)", e);
    }

    println!("\nInstalling Forge Standard Library...");
    if let Err(e) = forge(&["install", "foundry-rs/forge-std"]) {
        println!("  ─  Note: {} (skipping...)", e);
    }

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
        fs::write(".env", DOT_ENV).map_err(|e| format!("Failed to write .env: {e}"))?;
        println!("  ✔  .env");
    } else {
        println!("\n  ─  Skipping .env (already exists — not overwritten to protect secrets)");
    }

    println!("\nWriting .env.example...");
    fs::write(".env.example", DOT_ENV_EXAMPLE)
        .map_err(|e| format!("Failed to write .env.example: {e}"))?;
    println!("  ✔  .env.example");

    // Write .gitignore if it doesn't exist
    if !Path::new(".gitignore").exists() {
        println!("\nWriting .gitignore...");
        fs::write(".gitignore", GITIGNORE)
            .map_err(|e| format!("Failed to write .gitignore: {e}"))?;
        println!("  ✔  .gitignore");
    } else {
        println!("\n  ─  Skipping .gitignore (already exists)");
        append_gitignore_entry(".env")?;
    }

    println!("\nRunning forge build to verify configuration...");
    forge(&["build"])?;

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
