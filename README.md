# HARA CLI 🔨

**HARA** is a CLI scaffolding tool designed for Foundry projects. It simplifies the creation of upgradeable smart contracts using the **Diamond Storage Pattern** and standardizes the HARA development environment.

## Installation

### Option A — Linux / Debian 🐧

```bash
curl -fsSL https://raw.githubusercontent.com/meQlause/hara-cli/master/install/debian.sh | bash
```

This will:
1. Fetch the latest release from GitHub.
2. Install the binary to `~/.local/bin/hara`.
3. Automatically add `~/.local/bin` to your `PATH` in `~/.bashrc` or `~/.zshrc`.

### Option B — Windows PowerShell 🚀

**For PowerShell 7+ (Core):**
```powershell
irm https://raw.githubusercontent.com/meQlause/hara-cli/master/install/powershell.ps1 | iex
```

**For Windows PowerShell (5.1):**
```powershell
irm https://raw.githubusercontent.com/meQlause/hara-cli/master/install/win-powershell.ps1 | iex
```

This will:
1. Fetch the latest version via the GitHub API.
2. Install the binary to `$HOME\.local\bin\hara.exe`.
3. Fully automate the **User PATH** configuration and refresh the current session.

---

### Option C — Manual Download

1. Go to the [Releases page](https://github.com/meQlause/hara-cli/releases/latest) and download the binary for your OS.
2. Extract and move the binary:

**Linux:**
```bash
tar -xzf hara-linux-x86_64.tar.gz
mkdir -p ~/.local/bin
mv hara ~/.local/bin/
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

**Windows:**
```powershell
Expand-Archive -Path hara-windows-x86_64.zip -DestinationPath .
mkdir -Force "$HOME\.local\bin"
mv hara.exe "$HOME\.local\bin\"
```

3. Verify:
```bash
hara --version
```

---

## Commands

> [!IMPORTANT]
> HARA requires Unix-like utilities (`bash`, `curl`) and `forge` (Foundry) to be accessible in your PATH.

---

### `hara install` — Install Foundry

Installs Foundry (`forge`, `cast`, `anvil`) via the official installer script. Run this once on a fresh machine.

```bash
hara install
```

**What it does:**
1. Downloads the Foundry installer: `curl -fsSL https://foundry.paradigm.xyz | bash`
2. Runs `foundryup` to install all Foundry binaries.

---

### `hara init` — Initialise a HARA Project

Initialises a new Foundry project in the **current directory** with the HARA standard configuration.

```bash
mkdir my-contracts && cd my-contracts
hara init
```

**What it does:**
1. Runs `forge init` (skipped if `foundry.toml` already exists).
2. Installs dependencies:
   - `openzeppelin/openzeppelin-contracts@v5.0.1`
   - `openzeppelin/openzeppelin-contracts-upgradeable@v5.0.1`
   - `foundry-rs/forge-std`
3. Writes `foundry.toml` with the HARA standard configuration.
4. Writes `remappings.txt`.
5. Creates `.env` (from template, skipped if it exists) and `.env.example`.
6. Ensures `.env` is in `.gitignore`.
7. Runs `forge build` to verify the setup.

---

### `hara uc <ContractName>` — Scaffold an Upgradeable Contract

Scaffolds a complete upgradeable smart contract structure using the **Diamond Storage Pattern**.

> [!IMPORTANT]
> Must be run from the root of a Foundry project (where `foundry.toml` is located).

```bash
hara uc MyToken
```

**Generated file structure:**
```text
.
├── .github/workflows/
│   └── contract-limits.yml      # CI: size & gas enforcement
├── src/
│   ├── MyToken.sol              # V1 — main upgradeable contract
│   ├── MyTokenV2.sol            # V2 — upgrade-ready extension
│   └── libraries/
│       ├── MyTokenStorage.sol   # V1 Diamond Storage slot
│       ├── MyTokenV2Storage.sol # V2 Diamond Storage slot (separate slot)
│       ├── MyTokenView.sol      # View functions
│       ├── Structs.sol          # (Shared) Data structures
│       ├── Errors.sol           # (Shared) Custom errors
│       └── Events.sol           # (Shared) Events
├── script/
│   ├── DeployMyToken.s.sol      # Initial deploy via ERC-1967 proxy
│   └── UpgradeMyToken.s.sol     # Upgrade proxy to V2
└── test/
    ├── MyToken.t.sol            # Unit tests
    └── ContractLimits.t.sol     # CI: EIP-170 size & 300K gas checks
```

---

### Upgrading to V2

After deploying V1, use the generated upgrade script:

```bash
# Set env vars
export PRIVATE_KEY=<your_key>
export PROXY_ADDR=<deployed_proxy_address>

forge script script/UpgradeMyToken.s.sol \
  --rpc-url $HARA_RPC_URL \
  --broadcast \
  -vvvv
```

---

## Technical Details: Diamond Storage

HARA uses the Diamond Storage pattern so upgrades never corrupt existing state. Each contract version gets its own isolated storage slot:

```solidity
// V1
bytes32 constant myTokenPoint = keccak256("mytoken.storage");

// V2 — completely separate slot, zero collision risk
bytes32 constant myTokenV2Point = keccak256("mytoken.storage.v2");
```

---

Built with ❤️ for the Foundry ecosystem.
