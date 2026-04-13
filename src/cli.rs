use clap::{Parser, Subcommand};
use crate::commands;

/// hara — Foundry smart contract scaffolding tool
#[derive(Parser)]
#[command(
    name = "hara",
    version = "0.1.0",
    about = "Scaffold upgradeable smart contract structures for Foundry x HARA"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scaffold an Upgradeable Contract using the Diamond Storage Pattern
    Uc {
        /// Contract name in PascalCase (e.g. MyToken)
        name: String,
    },
}

pub fn run() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Uc { name } => commands::uc::run(&name),
    };

    if let Err(e) = result {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}
