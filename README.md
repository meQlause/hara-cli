# HARA CLI 🔨

**HARA** is a CLI scaffolding tool designed for Foundry projects. It simplifies the creation of upgradeable smart contracts using the **Diamond Storage Pattern** and standardizes the HARA development environment.

## Installation

### From GitHub Releases
Download the latest binary for your operating system from the [Releases](https://github.com/meQlause/hara-cli/releases) page.

- **Linux**: Download `hara-linux-x86_64.tar.gz`, extract, and move to `/usr/local/bin`:
  ```bash
  tar -xzf hara-linux-x86_64.tar.gz
  sudo mv hara /usr/local/bin/
  ```
- **Debian/Ubuntu**: Download `hara.deb` and install:
  ```bash
  sudo dpkg -i hara.deb
  ```
- **Windows (Git Bash)**: Download `hara-windows-x86_64.zip`, extract `hara.exe`, and add its directory to PATH.

---

## Commands

> [!IMPORTANT]
> HARA assumes **Linux or Git Bash** as the shell environment.
> `hara install` and `hara init` require `bash`, `curl`, and `forge` to be accessible in your PATH.

---

### `hara install` — Install Foundry

Installs Foundry (`forge`, `cast`, `anvil`) via the official installer script. Run this once on a fresh machine.

```bash
hara install
```

**What it does:**
1. Downloads the Foundry installer: `curl -fsSL https://foundry.paradigm.xyz | bash`
2. Runs `foundryup` to install all Foundry binaries.

> [!NOTE]
> After installation you may need to restart your terminal for `forge` to be available in PATH. If so, just run `foundryup` manually.

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

**Generated `foundry.toml`:**
```toml
[profile.default]
evm_version    = "paris"
src            = "src"
out            = "out"
libs           = ["lib"]
solc_version   = "0.8.29"
optimizer      = true
optimizer_runs = 200

fs_permissions = [{ access = "read", path = "." }]

[rpc_endpoints]
hara = "${HARA_RPC_URL}"

[profile.ci]
fuzz.runs   = 512
gas_reports = ["*"]
```

**Generated `remappings.txt`:**
```
@openzeppelin/=lib/openzeppelin-contracts/
@openzeppelin-upgradeable/=lib/openzeppelin-contracts-upgradeable/
@forge-std/=lib/forge-std/
forge-std/=lib/forge-std/src/
ds-test/=lib/forge-std/lib/ds-test/src/
```

After `hara init` completes, fill in your `PRIVATE_KEY` in `.env` and you are ready to scaffold contracts.

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

**Scaffolding behaviour:**
- Prompts whether to reset `src/`, `script/`, `test/` before writing.
- Contract files (V1, V2, Storage, View, Scripts, Tests) are **always overwritten**.
- Shared libraries (`Structs.sol`, `Errors.sol`, `Events.sol`) are **created once and never overwritten**.

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

The script atomically:
1. Deploys the new `MyTokenV2` implementation.
2. Calls `upgradeToAndCall` on the proxy with the `initializeV2` calldata.

---

## CI Pipeline

`hara uc` automatically generates `.github/workflows/contract-limits.yml` which:
- Runs on every `push` and `pull_request`.
- Checks all contract sizes are **< 24,576 bytes** (EIP-170).
- Runs `ContractLimitsTest` to enforce **< 300,000 gas** per public call.
- Runs the full `forge test` suite.
- Generates and uploads a gas snapshot artifact.

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

## Roadmap & Future Commands 🚀

HARA is actively maintained with new scaffolding templates added continuously. Planned commands:

- `hara fc <Name>` — Scaffold a **Factory Contract** structure.
- Additional Diamond-compatible utility generators.

---

Built with ❤️ for the Foundry ecosystem. HARA is under constant maintenance to ensure compatibility with the latest Foundry and Solidity standards.
