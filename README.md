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
3. Automatically add `~/.local/bin` to your `PATH`.

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
2. Extract and move the binary to your desired local bin directory.

---

## Commands

### `hara foundry install` — Install Foundry

Installs the official Foundry toolchain (`forge`, `cast`, `anvil`).

```bash
hara foundry install
```

**What it does:**
- **Windows**: Uses native **PowerShell** to download and extract Foundry directly from GitHub (Git Bash is not required).
- **Linux**: Downloads the Foundry installer via `curl` and runs the official shell script.

---

### `hara foundry init` — Initialise a HARA Project

Initialises a new Foundry project in the **current directory** with the HARA standard configuration.

```bash
mkdir my-contracts && cd my-contracts
hara foundry init
```

**What it does:**
1. Runs `forge init`.
2. Installs dependencies: OpenZeppelin Contracts (v5.0.1) and Forge Standard Library.
3. Writes HARA-standard `foundry.toml`, `remappings.txt`, and `.env`.
4. Runs `forge build` to verify the setup.

---

### `hara foundry contract uc <ContractName>` — Scaffold an Upgradeable Contract

Scaffolds a complete upgradeable smart contract structure using the **Diamond Storage Pattern**.

> [!IMPORTANT]
> Must be run from the root of a Foundry project (where `foundry.toml` is located).

```bash
hara foundry contract uc MyToken
```

**Generated file structure:**
```text
.
├── src/
│   ├── MyToken.sol              # V1 — main upgradeable contract
│   ├── MyTokenV2.sol            # V2 — upgrade-ready extension
│   └── libraries/
│       ├── MyTokenStorage.sol   # V1 Diamond Storage slot
│       ├── MyTokenV2Storage.sol # V2 Diamond Storage slot
...
```

---

## Technical Details: Diamond Storage

HARA uses the Diamond Storage pattern so upgrades never corrupt existing state. Each contract version gets its own isolated storage slot.

---

Built with ❤️ for the Foundry ecosystem.
