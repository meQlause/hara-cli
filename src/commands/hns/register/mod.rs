pub mod client;
pub mod ops;
pub mod utils;

use eyre::{Result, Context};
use alloy::{
    network::EthereumWallet,
    primitives::Address,
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};
use crate::utils::config;

/// Orchestrates the HNS contract registration process.
pub async fn run(arg: &str) -> Result<()> {
    dotenv::dotenv().ok();

    let rpc = config::get_or_prompt(
        "HARA_RPC",
        "HARA RPC URL",
        "http://20.198.228.24:5625"
    )?;

    let pk_raw = config::get_or_prompt(
        "HARA_PK",
        "Private Key",
        "your_private_key_without_0x"
    )?;
    let pk = pk_raw.trim_start_matches("0x");

    let proxy_addr: Address = config::get_or_prompt(
        "HARA_HNS_PROXY",
        "HNS Proxy Address",
        "0x0000000000000000000000000000000000000000"
    )?
    .parse()
    .wrap_err("Invalid HARA_HNS_PROXY address format")?;

    let signer: PrivateKeySigner = pk.parse()
        .wrap_err("Invalid private key (HARA_PK) format")?;

    let owner  = signer.address();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect(&rpc)
        .await
        .wrap_err_with(|| format!("Failed to connect to RPC: {}", rpc))?;

    let registry = client::HnsRegistry::new(provider, proxy_addr, owner);

    ops::process_path(arg, &registry).await?;

    Ok(())
}

/// Forcefully re-prompts for environment configurations (RPC, PK, Proxy) and updates .env.
pub async fn reset() -> Result<()> {
    tracing::info!("Resetting HNS Register configurations...");

    let rpc = config::prompt(
        "HARA RPC URL",
        "http://20.198.228.24:5625"
    )?;
    config::update_env("HARA_RPC", &rpc)?;

    let pk = config::prompt(
        "Private Key (without 0x)",
        "your_private_key"
    )?;
    config::update_env("HARA_PK", &pk)?;

    let proxy = config::prompt(
        "HNS Proxy Address",
        "0x..."
    )?;
    config::update_env("HARA_HNS_PROXY", &proxy)?;

    tracing::info!("Configurations reset successfully. You can now run 'register' normally.");
    Ok(())
}
