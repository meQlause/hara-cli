use clap::{Parser, Subcommand, Args};
use crate::commands;

/// HARA CLI - A scaffolding and registration tool for Foundry projects.
#[derive(Parser)]
#[command(
    name = "hara",
    version,
    about = "Scaffold upgradeable smart contract structures for Foundry x HARA"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// The set of high-level subcommands supported by the HARA CLI.
#[derive(Subcommand)]
pub enum Commands {
    /// Foundry-related utilities (init, install, contract).
    Foundry(FoundryArgs),

    /// HARA Network Solution (HNS) registry and ABI tooling.
    Hns(HnsArgs),
}

/// Arguments for HNS operations.
#[derive(Args)]
pub struct HnsArgs {
    #[command(subcommand)]
    pub command: HnsCommands,
}

/// Subcommands for the HNS registry and ABI tooling.
#[derive(Subcommand)]
pub enum HnsCommands {
    /// Register contract addresses and ABIs to the HNS registry.
    Register(RegisterArgs),

    /// Encode, decode, or inspect ABI binary files.
    Codex(CodexArgs),
}

/// Arguments for Register operations.
#[derive(Args)]
pub struct RegisterArgs {
    #[command(subcommand)]
    pub command: Option<RegisterSubcommands>,

    /// Path to a JSON schema file or a directory containing JSON schema files.
    pub path: Option<String>,
}

/// Subcommands for the HNS Register.
#[derive(Subcommand)]
pub enum RegisterSubcommands {
    /// Re-prompt for environment configurations (RPC, PK, Proxy Address).
    Reset,
}

/// Arguments for Codex operations.
#[derive(Args)]
pub struct CodexArgs {
    #[command(subcommand)]
    pub command: CodexSubcommands,
}

/// Subcommands for the ABI Codex.
#[derive(Subcommand)]
pub enum CodexSubcommands {
    /// Encode JSON ABI files into compact binary (.abi.bin) format.
    Encode {
        /// Path to a .json file or a directory of .json files to encode.
        target: String,
    },

    /// Decode a compact binary (.abi.bin) file back to JSON.
    Decode {
        /// Path to the .abi.bin file to decode.
        path: String,
    },

    /// Inspect a compact binary (.abi.bin) file with a detailed hex dump.
    Inspect {
        /// Path to the .abi.bin file to inspect.
        path: String,
    },
}

/// Arguments for Foundry-related operations.
#[derive(Args)]
pub struct FoundryArgs {
    #[command(subcommand)]
    pub command: FoundryCommands,
}

/// Subcommands for the Foundry toolchain and contract scaffolding.
#[derive(Subcommand)]
pub enum FoundryCommands {
    /// Initialise a HARA-standard Foundry project.
    Init,

    /// Install the official Foundry toolchain (forge, cast, anvil).
    Install,

    /// Scaffold new smart contract structures (uc, uups, fc).
    Contract(ContractArgs),
}

/// Arguments for contract scaffolding operations.
#[derive(Args)]
pub struct ContractArgs {
    #[command(subcommand)]
    pub command: ContractCommands,
}

/// Specific contract templates supported for scaffolding.
#[derive(Subcommand)]
pub enum ContractCommands {
    /// Scaffold an Upgradeable Contract using the Diamond Storage pattern.
    Uc {
        /// Contract name in PascalCase (e.g., MyToken).
        name: String,
    },
}

/// Parses command-line arguments and orchestrates command execution.
pub async fn run() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Hns(hns_args) => match hns_args.command {
            HnsCommands::Register(args) => {
                if let Some(RegisterSubcommands::Reset) = args.command {
                    commands::hns::register::reset().await.map_err(|e| e.to_string())
                } else if let Some(path) = args.path {
                    commands::hns::register::run(&path).await.map_err(|e| e.to_string())
                } else {
                    // Default to current directory if no path and no subcommand
                    commands::hns::register::run(".").await.map_err(|e| e.to_string())
                }
            }
            HnsCommands::Codex(args) => {
                commands::hns::codex::run(args)
            }
        },
        Commands::Foundry(foundry_args) => match foundry_args.command {
            FoundryCommands::Init    => commands::foundry::init::run(),
            FoundryCommands::Install => commands::foundry::install::run(),
            FoundryCommands::Contract(contract_args) => match contract_args.command {
                ContractCommands::Uc { name } => commands::foundry::contract::uc::run(&name),
            },
        },
    };

    if let Err(e) = result {
        tracing::error!("Error: {}", e);
        std::process::exit(1);
    }
}
