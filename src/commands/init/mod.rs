use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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

// ── Forge binary resolution ────────────────────────────────────────────────────

/// Finds the `forge` binary, checking the Foundry install directory first
/// before falling back to whatever is on PATH.
/// Works on Windows (Git Bash & native), Linux, and macOS.
fn forge_bin() -> PathBuf {
    let home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_default();

    let mut candidates: Vec<PathBuf> = vec![
        // Standard Foundry install location on all platforms
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

/// Runs a forge subcommand with the resolved binary, streaming I/O.
fn forge(args: &[&str]) -> Result<(), String> {
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

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run() -> Result<(), String> {
    println!("🚀 Initialising HARA Foundry project...\n");

    // ── 1. forge init ──────────────────────────────────────────────────────────
    if Path::new("foundry.toml").exists() {
        println!("  ─  Skipping forge init (foundry.toml already exists)");
    } else {
        println!("📦 Running forge init...");
        // --force allows initialising in a non-empty directory.
        // --no-git skips git init (we don't require git for scaffolding).
        // If --no-git fails (older forge), retry without it.
        let result = forge(&["init", "--force", "--no-git"]);
        if result.is_err() {
            forge(&["init", "--force"])?;
        }
    }

    // ── 2. Install OpenZeppelin dependencies ───────────────────────────────────
    println!("\n📥 Installing OpenZeppelin Contracts v5.0.1...");
    forge(&[
        "install",
        "openzeppelin/openzeppelin-contracts@v5.0.1",
    ])?;

    println!("\n📥 Installing OpenZeppelin Upgradeable Contracts v5.0.1...");
    forge(&[
        "install",
        "openzeppelin/openzeppelin-contracts-upgradeable@v5.0.1",
    ])?;

    println!("\n📥 Installing Forge Standard Library...");
    forge(&["install", "foundry-rs/forge-std", "--no-commit"])?;

    // ── 3. Write config files ──────────────────────────────────────────────────
    println!("\n📝 Writing foundry.toml (HARA standard)...");
    fs::write("foundry.toml", FOUNDRY_TOML)
        .map_err(|e| format!("Failed to write foundry.toml: {e}"))?;
    println!("  ✔  foundry.toml");

    println!("\n📝 Writing remappings.txt...");
    fs::write("remappings.txt", REMAPPINGS)
        .map_err(|e| format!("Failed to write remappings.txt: {e}"))?;
    println!("  ✔  remappings.txt");

    if !Path::new(".env").exists() {
        println!("\n📝 Writing .env...");
        fs::write(".env", DOT_ENV).map_err(|e| format!("Failed to write .env: {e}"))?;
        println!("  ✔  .env");
    } else {
        println!("\n  ─  Skipping .env (already exists — not overwritten to protect secrets)");
    }

    println!("\n📝 Writing .env.example...");
    fs::write(".env.example", DOT_ENV_EXAMPLE)
        .map_err(|e| format!("Failed to write .env.example: {e}"))?;
    println!("  ✔  .env.example");

    append_gitignore_entry(".env")?;

    // ── 4. Verify build ────────────────────────────────────────────────────────
    println!("\n🔨 Running forge build to verify configuration...");
    forge(&["build"])?;

    println!("\n✅ HARA project ready!");
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
