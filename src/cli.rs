use clap::{Parser, Subcommand};
use crate::commands;

/// hara — Foundry smart contract scaffolding tool
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

#[derive(Subcommand)]
pub enum Commands {
    /// Install Foundry (forge, cast, anvil) via the official installer — Linux / Git Bash only
    Install,

    /// Initialise a HARA-standard Foundry project in the current directory
    Init,

    /// Scaffold an Upgradeable Contract using the Diamond Storage Pattern
    Uc {
        /// Contract name in PascalCase (e.g. MyToken)
        name: String,
    },
}

pub fn run() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Install          => commands::install::run(),
        Commands::Init             => commands::init::run(),
        Commands::Uc { name }      => commands::uc::run(&name),
    };

    if let Err(e) = result {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}
