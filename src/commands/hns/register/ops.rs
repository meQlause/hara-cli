use std::fs;
use std::path::{Path, PathBuf};
use eyre::{Result, Context, eyre};
use serde::Deserialize;
use alloy::primitives::Address;
use alloy::providers::Provider;
use super::client::{HnsRegistry, content_type};
use super::utils;

/// Internal schema for HARA Network Solution (HNS) JSON registration files.
#[derive(Deserialize)]
struct Schema {
    abi: serde_json::Value,
    contract_address: String,
}

/// Recursively processes a path (file or directory) to register contract schemas.
///
/// Supports `.json` files (JSON ABI) and `.bin` files (raw bytecode).
pub async fn process_path<P: Provider>(arg: &str, registry: &HnsRegistry<P>) -> Result<()> {
    let p = Path::new(arg);

    if p.is_dir() {
        tracing::info!("Processing directory: {}", arg);
        let entries = fs::read_dir(p)
            .wrap_err_with(|| format!("Failed to read directory: {}", arg))?;

        for entry in entries {
            let fp = entry?.path();
            match fp.extension().and_then(|x| x.to_str()) {
                Some("json") => process_json_file(&fp, registry).await?,
                Some("bin")  => process_bin_file(&fp, registry).await?,
                _ => {}
            }
        }
    } else {
        match p.extension().and_then(|x| x.to_str()) {
            Some("json") => process_json_file(p, registry).await?,
            Some("bin")  => process_bin_file(p, registry).await?,
            _ => eyre::bail!("Unsupported file type. Provide a .json or .bin file."),
        }
    }

    Ok(())
}

/// Parses a JSON schema file and registers the contract ABI with the HNS registry.
async fn process_json_file<P: Provider>(path: &Path, registry: &HnsRegistry<P>) -> Result<()> {
    let label = utils::extract_label(path)?;

    let content = fs::read_to_string(path)
        .wrap_err_with(|| format!("Failed to read file: {:?}", path))?;

    let s: Schema = serde_json::from_str(&content)
        .wrap_err_with(|| format!("Failed to parse JSON schema: {:?}", path))?;

    let addr: Address = s.contract_address.parse()
        .wrap_err_with(|| format!("Invalid contract address in schema: {}", s.contract_address))?;

    let abi_bytes = serde_json::to_vec(&s.abi)
        .wrap_err("Failed to serialize ABI to JSON")?;

    tracing::info!("Registering ABI (JSON, contentType=1): {}", label);
    registry.register_contract(&label, addr, abi_bytes, content_type::JSON).await?;

    Ok(())
}

/// Reads a raw `.bin` file and registers its bytes as bytecode with the HNS registry.
///
/// Looks for a sidecar `.json` with matching stem to supply `contract_address`.
async fn process_bin_file<P: Provider>(path: &Path, registry: &HnsRegistry<P>) -> Result<()> {
    let label = utils::extract_label(path)?;

    let sidecar_candidates = [
        path.with_extension("json"),
        PathBuf::from(path.to_string_lossy().replace(".abi.bin", ".json")),
    ];

    let mut sidecar_content = None;
    let mut found_path      = None;

    for candidate in &sidecar_candidates {
        if let Ok(content) = fs::read_to_string(candidate) {
            sidecar_content = Some(content);
            found_path      = Some(candidate.clone());
            break;
        }
    }

    let sidecar_content = sidecar_content.ok_or_else(|| {
        eyre!("Missing sidecar JSON for .bin file.\nChecked: {:?}", sidecar_candidates)
    })?;

    let s: Schema = serde_json::from_str(&sidecar_content)
        .wrap_err_with(|| format!("Failed to parse sidecar JSON: {:?}", found_path))?;

    let addr: Address = s.contract_address.parse()
        .wrap_err_with(|| format!("Invalid contract address in sidecar: {}", s.contract_address))?;

    let bin_bytes = fs::read(path)
        .wrap_err_with(|| format!("Failed to read .bin file: {:?}", path))?;

    tracing::info!("Registering bytecode (binary, contentType=8): {}", label);
    registry.register_contract(&label, addr, bin_bytes, content_type::BIN).await?;

    Ok(())
}
