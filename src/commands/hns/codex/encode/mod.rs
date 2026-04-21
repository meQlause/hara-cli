use std::fs;
use std::path::{Path, PathBuf};
use tiny_keccak::{Hasher, Keccak};
use super::parser::parse_abi_json;

pub const T_UINT256: u8 = 0x01;
pub const T_ADDRESS: u8 = 0x02;
pub const T_BOOL: u8 = 0x03;
pub const T_BYTES32: u8 = 0x04;
pub const T_STRING: u8 = 0x05;
pub const T_BYTES: u8 = 0x06;
pub const T_UINT8: u8  = 0x07;
pub const T_UINT16: u8 = 0x08;
pub const T_UINT32: u8 = 0x09;
pub const T_UINT64: u8 = 0x0A;
pub const T_UINT128: u8 = 0x0B;
/// `0x0C <n>` — bytesN where n ∈ 1..=4.
pub const T_BYTES_N: u8 = 0x0C;
pub const MOD_ARRAY: u8 = 0x20;
pub const MOD_TUPLE: u8 = 0x30;
pub const ENTRY_FUNCTION: u8 = 0x01;
pub const ENTRY_EVENT: u8 = 0x02;
pub const ENTRY_ERROR: u8 = 0x03;

#[derive(Debug, Clone, PartialEq)]
pub enum AbiType {
    Uint256,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint128,
    Address,
    Bool,
    Bytes32,
    /// Parameterized bytesN, n ∈ 1..=4.
    BytesN(u8),
    AbiString,
    Bytes,
    Array(Box<AbiType>),
    Tuple(Vec<AbiType>),
}

impl AbiType {
    pub fn canonical(&self) -> String {
        match self {
            AbiType::Uint256   => "uint256".into(),
            AbiType::Uint8     => "uint8".into(),
            AbiType::Uint16    => "uint16".into(),
            AbiType::Uint32    => "uint32".into(),
            AbiType::Uint64    => "uint64".into(),
            AbiType::Uint128   => "uint128".into(),
            AbiType::Address   => "address".into(),
            AbiType::Bool      => "bool".into(),
            AbiType::Bytes32   => "bytes32".into(),
            AbiType::BytesN(n) => format!("bytes{}", n),
            AbiType::AbiString => "string".into(),
            AbiType::Bytes     => "bytes".into(),
            AbiType::Array(inner) => format!("{}[]", inner.canonical()),
            AbiType::Tuple(fields) => {
                let parts: Vec<String> = fields.iter().map(|f| f.canonical()).collect();
                format!("({})", parts.join(","))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AbiFunction {
    pub name: Vec<u8>,
    pub inputs: Vec<AbiType>,
    pub outputs: Vec<AbiType>,
}

#[derive(Debug, Clone)]
pub struct EventParam {
    pub ty: AbiType,
    pub indexed: bool,
}

#[derive(Debug, Clone)]
pub struct AbiEvent {
    pub name: Vec<u8>,
    pub params: Vec<EventParam>,
}

#[derive(Debug, Clone)]
pub struct AbiError {
    pub name: Vec<u8>,
    pub params: Vec<AbiType>,
}

#[derive(Debug, Clone)]
pub struct AbiBlob {
    pub functions: Vec<AbiFunction>,
    pub events: Vec<AbiEvent>,
    pub errors: Vec<AbiError>,
}

/// The command entry point for encoding JSON ABIs to binary.
pub fn run(target: &str) -> Result<(), String> {
    let path = Path::new(target);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", target));
    }

    let files = collect_json_files(path);

    if files.is_empty() {
        return Err(format!("No .json files found in: {}", target));
    }

    tracing::info!("Found {} file(s) to encode", files.len());

    let mut ok   = 0usize;
    let mut fail = 0usize;

    for file_path in &files {
        match process_encode(file_path) {
            Ok(bytes) => {
                let out = output_path(file_path);
                tracing::info!(
                    "Encoded: {} -> {} ({} bytes)",
                    file_path.display(),
                    out.file_name().unwrap_or_default().to_string_lossy(),
                    bytes
                );
                ok += 1;
            }
            Err(e) => {
                tracing::error!("Failed: {} -> {}", file_path.display(), e);
                fail += 1;
            }
        }
    }

    tracing::info!("Done: {} encoded, {} failed", ok, fail);

    if fail > 0 {
        return Err(format!("{} file(s) failed to encode", fail));
    }

    Ok(())
}

fn process_encode(path: &Path) -> Result<usize, String> {
    let json = fs::read_to_string(path)
        .map_err(|e| format!("Read error: {}", e))?;

    let blob    = parse_abi_json(&json)?;
    let encoded = encode_blob(&blob);
    let size    = encoded.len();

    let out_path = output_path(path);
    fs::write(&out_path, &encoded)
        .map_err(|e| format!("Write error: {}", e))?;

    Ok(size)
}

fn output_path(src: &Path) -> PathBuf {
    let stem = src.file_stem().unwrap_or_default().to_string_lossy();
    src.with_file_name(format!("{}.abi.bin", stem))
}

fn collect_json_files(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_recursive(path, &mut files);
    files
}

fn collect_recursive(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_file() {
        let name = path.to_string_lossy();
        if name.ends_with(".abi.bin") {
            return;
        }
        if path.extension().map_or(false, |e| e.eq_ignore_ascii_case("json")) {
            out.push(path.to_path_buf());
        }
    } else if path.is_dir() {
        let mut entries: Vec<PathBuf> = fs::read_dir(path)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .collect();
        entries.sort();
        for child in entries {
            collect_recursive(&child, out);
        }
    }
}

pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut k = Keccak::v256();
    k.update(data);
    let mut out = [0u8; 32];
    k.finalize(&mut out);
    out
}

fn selector(sig: &str) -> [u8; 4] {
    let h = keccak256(sig.as_bytes());
    [h[0], h[1], h[2], h[3]]
}

fn fn_sig(name: &[u8], inputs: &[AbiType]) -> String {
    let n = std::str::from_utf8(name).unwrap_or("unknown");
    let params: Vec<String> = inputs.iter().map(|t| t.canonical()).collect();
    format!("{}({})", n, params.join(","))
}

fn error_sig(name: &[u8], params: &[AbiType]) -> String {
    let n = std::str::from_utf8(name).unwrap_or("unknown");
    let parts: Vec<String> = params.iter().map(|t| t.canonical()).collect();
    format!("{}({})", n, parts.join(","))
}

pub fn encode_type(ty: &AbiType, buf: &mut Vec<u8>) {
    match ty {
        AbiType::Uint256   => buf.push(T_UINT256),
        AbiType::Uint8     => buf.push(T_UINT8),
        AbiType::Uint16    => buf.push(T_UINT16),
        AbiType::Uint32    => buf.push(T_UINT32),
        AbiType::Uint64    => buf.push(T_UINT64),
        AbiType::Uint128   => buf.push(T_UINT128),
        AbiType::Address   => buf.push(T_ADDRESS),
        AbiType::Bool      => buf.push(T_BOOL),
        AbiType::Bytes32   => buf.push(T_BYTES32),
        AbiType::BytesN(n) => { buf.push(T_BYTES_N); buf.push(*n); }
        AbiType::AbiString => buf.push(T_STRING),
        AbiType::Bytes     => buf.push(T_BYTES),
        AbiType::Array(inner) => {
            buf.push(MOD_ARRAY);
            encode_type(inner, buf);
        }
        AbiType::Tuple(fields) => {
            buf.push(MOD_TUPLE);
            buf.push(fields.len() as u8);
            for f in fields {
                encode_type(f, buf);
            }
        }
    }
}

fn encode_function(f: &AbiFunction) -> Vec<u8> {
    let sig = fn_sig(&f.name, &f.inputs);
    let sel = selector(&sig);
    let mut buf = Vec::new();
    buf.extend_from_slice(&sel);
    buf.push(f.name.len() as u8);
    buf.extend_from_slice(&f.name);
    buf.push(f.inputs.len() as u8);
    for t in &f.inputs {
        encode_type(t, &mut buf);
    }
    buf.push(f.outputs.len() as u8);
    for t in &f.outputs {
        encode_type(t, &mut buf);
    }
    buf
}

fn encode_event(e: &AbiEvent) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.push(e.name.len() as u8);
    buf.extend_from_slice(&e.name);
    buf.push(e.params.len() as u8);
    for p in &e.params {
        buf.push(if p.indexed { 0x01 } else { 0x00 });
        encode_type(&p.ty, &mut buf);
    }
    buf
}

fn encode_error(e: &AbiError) -> Vec<u8> {
    let sig = error_sig(&e.name, &e.params);
    let sel = selector(&sig);
    let mut buf = Vec::new();
    buf.extend_from_slice(&sel);
    buf.push(e.name.len() as u8);
    buf.extend_from_slice(&e.name);
    buf.push(e.params.len() as u8);
    for t in &e.params {
        encode_type(t, &mut buf);
    }
    buf
}

pub fn encode_blob(blob: &AbiBlob) -> Vec<u8> {
    let total = blob.functions.len() + blob.events.len() + blob.errors.len();
    let mut out = Vec::new();
    out.push(total as u8);
    for f in &blob.functions {
        let data = encode_function(f);
        out.push(ENTRY_FUNCTION);
        out.extend_from_slice(&(data.len() as u16).to_be_bytes());
        out.extend_from_slice(&data);
    }
    for e in &blob.events {
        let data = encode_event(e);
        out.push(ENTRY_EVENT);
        out.extend_from_slice(&(data.len() as u16).to_be_bytes());
        out.extend_from_slice(&data);
    }
    for e in &blob.errors {
        let data = encode_error(e);
        out.push(ENTRY_ERROR);
        out.extend_from_slice(&(data.len() as u16).to_be_bytes());
        out.extend_from_slice(&data);
    }
    out
}
