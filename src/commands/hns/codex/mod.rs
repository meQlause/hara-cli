pub mod encode;
pub mod decode;
pub mod inspect;
pub mod parser;
pub mod bin;
#[cfg(test)] mod tests;

use crate::cli::{CodexArgs, CodexSubcommands};
use eyre::Result;

/// Orchestrates the ABI Codex commands: encode, decode, and inspect.
pub fn run(args: CodexArgs) -> Result<(), String> {
    match args.command {
        CodexSubcommands::Encode { target } => encode::run(&target),
        CodexSubcommands::Decode { path }   => decode::run(&path),
        CodexSubcommands::Inspect { path }  => inspect::run(&path),
        CodexSubcommands::Bin { path }      => bin::run(&path),
    }
}
