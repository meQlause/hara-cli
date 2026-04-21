use std::fs;

/// Standard HARA foundry.toml template.
pub const FOUNDRY_TOML: &str = r#"[profile.default]
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
hara = "${HARA_RPC}"

[profile.ci]
fuzz.runs   = 512
gas_reports = ["*"]
"#;

/// Standard HARA remappings template.
pub const REMAPPINGS: &str = "@openzeppelin/=lib/openzeppelin-contracts/
@openzeppelin-upgradeable/=lib/openzeppelin-contracts-upgradeable/
@forge-std/=lib/forge-std/
forge-std/=lib/forge-std/src/
ds-test/=lib/forge-std/lib/ds-test/src/
";

/// Standard HARA .env template.
pub const DOT_ENV: &str = "# Private key for deployment (without 0x prefix)
HARA_PK=your_private_key_here

# HARA RPC URL
HARA_RPC=http://20.198.228.24:5625

# HARA HNS Proxy address
HARA_HNS_PROXY=0x0000000000000000000000000000000000000000

# Optional: Gas configuration
GAS_LIMIT=30000000
GAS_PRICE=0
";

/// Standard HARA .env.example template.
pub const DOT_ENV_EXAMPLE: &str = "# Private key for deployment (without 0x prefix)
HARA_PK=<your_private_key_without_0x>

# HARA RPC URL
HARA_RPC=http://20.198.228.24:5625

# HARA HNS Proxy address
HARA_HNS_PROXY=0x0000000000000000000000000000000000000000

# Optional: Gas configuration
GAS_LIMIT=30000000
GAS_PRICE=0
";

/// Standard HARA .gitignore template.
pub const GITIGNORE: &str = r#"# Foundry
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

/// Appends an entry to .gitignore if it doesn't already exist.
pub fn append_gitignore_entry(entry: &str) -> Result<(), String> {
    let path = std::path::Path::new(".gitignore");
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
        tracing::info!("  -  Added '{entry}' to .gitignore");
    }
    Ok(())
}
