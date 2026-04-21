use std::fs;
use std::path::Path;
use super::decode::{decode_blob, blob_to_json};

/// The command entry point for inspecting compact binary (.abi.bin) files.
pub fn run(path_str: &str) -> Result<(), String> {
    let path = Path::new(path_str);
    if !path.exists() {
        return Err(format!("File not found: {}", path_str));
    }

    let data = fs::read(path).map_err(|e| format!("Read error: {}", e))?;
    
    // Header
    println!("--- ABI Binary Inspection: {} ---", path.display());
    println!("File size: {} bytes", data.len());
    
    // Hex Dump
    println!("\nHex Dump:");
    print_hexdump(&data);

    // Decode and summary
    match decode_blob(&data) {
        Ok(blob) => {
            println!("\nSummary:");
            println!("  Functions: {}", blob.functions.len());
            println!("  Events:    {}", blob.events.len());
            println!("  Errors:    {}", blob.errors.len());

            println!("\nJSON Manifest:");
            let json = blob_to_json(&blob);
            println!("{}", pretty_json(&json));
        }
        Err(e) => {
            tracing::error!("Failed to decode ABI body: {}", e);
        }
    }

    Ok(())
}

fn print_hexdump(data: &[u8]) {
    for (i, chunk) in data.chunks(16).enumerate() {
        print!("{:06x}  ", i * 16);
        for (j, byte) in chunk.iter().enumerate() {
            print!("{:02x} ", byte);
            if j == 7 { print!(" "); }
        }
        // padding
        if chunk.len() < 16 {
            for j in chunk.len()..16 {
                print!("   ");
                if j == 7 { print!(" "); }
            }
        }
        print!(" |");
        for byte in chunk {
            if byte.is_ascii_graphic() || *byte == b' ' {
                print!("{}", *byte as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }
}

fn pretty_json(json: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(json) {
        if let Ok(s) = serde_json::to_string_pretty(&v) {
            return s;
        }
    }
    json.to_string()
}
