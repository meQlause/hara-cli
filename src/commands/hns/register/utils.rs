use alloy::primitives::{keccak256, FixedBytes};
use std::path::Path;
use eyre::{Result, eyre};

/// Calculates the namehash for a given label under the 'hara.ethnet' namespace.
pub fn calc_node(label: &str) -> FixedBytes<32> {
    let empty_node  = FixedBytes::<32>::ZERO;
    let ethnet_node = keccak256([empty_node.as_slice(), keccak256("ethnet").as_slice()].concat());
    let hara_node   = keccak256([ethnet_node.as_slice(), keccak256("hara").as_slice()].concat());
    keccak256([hara_node.as_slice(), keccak256(label).as_slice()].concat())
}

/// Calculates the namehash for the parent 'hara.ethnet' node.
pub fn calc_parent_node() -> FixedBytes<32> {
    let empty_node  = FixedBytes::<32>::ZERO;
    let ethnet_node = keccak256([empty_node.as_slice(), keccak256("ethnet").as_slice()].concat());
    keccak256([ethnet_node.as_slice(), keccak256("hara").as_slice()].concat())
}

/// Extracts and normalizes the HNS subnode label from a file path.
///
/// Converts to lowercase and strips all non-alphanumeric characters.
pub fn extract_label(path: &Path) -> Result<String> {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| eyre!("Invalid file name: {:?}", path))?;

    let normalized: String = stem
        .trim_end_matches(".hara.ethnet")
        .trim_end_matches(".hara")
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();

    if normalized.is_empty() {
        return Err(eyre!("Extracted label is empty after normalization for path: {:?}", path));
    }

    Ok(normalized)
}
