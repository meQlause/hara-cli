use std::fs;
use std::path::Path;
use eyre::Result;

/// The command entry point for converting hex text files to binary files.
pub fn run(path_str: &str) -> Result<(), String> {
    let path = Path::new(path_str);
    if !path.exists() {
        return Err(format!("File not found: {}", path_str));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Read error: {}", e))?;
    
    let hex_trimmed = content.trim();
    let hex_clean: String = hex_trimmed
        .strip_prefix("0x")
        .unwrap_or(hex_trimmed)
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect();
    
    let bytes = alloy::hex::decode(&hex_clean)
        .map_err(|e| format!("Hex decode error: {}. Make sure the file only contains a valid hex string (with or without 0x prefix).", e))?;

    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let out_path = path.with_file_name(format!("{}.abi.bin", stem));
    
    fs::write(&out_path, &bytes)
        .map_err(|e| format!("Write error: {}", e))?;

    tracing::info!("Converted: {} -> {} ({} bytes)", path.display(), out_path.display(), bytes.len());
    println!("Success: Written {} bytes to {}", bytes.len(), out_path.display());

    Ok(())
}
