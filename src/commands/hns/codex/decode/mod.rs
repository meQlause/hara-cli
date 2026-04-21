use std::fs;
use std::path::Path;
use super::encode::{
    AbiBlob, AbiError, AbiEvent, AbiFunction, AbiType, EventParam,
    ENTRY_ERROR, ENTRY_EVENT, ENTRY_FUNCTION, MOD_ARRAY, MOD_TUPLE,
    T_ADDRESS, T_BOOL, T_BYTES, T_BYTES32, T_STRING, T_UINT256,
    T_UINT8, T_UINT16, T_UINT32, T_UINT64, T_UINT128, T_BYTES_N,
};

pub struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read_byte(&mut self) -> Result<u8, &'static str> {
        if self.pos >= self.data.len() {
            return Err("unexpected end of data");
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], &'static str> {
        if self.pos + n > self.data.len() {
            return Err("unexpected end of data");
        }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    fn read_u16_be(&mut self) -> Result<u16, &'static str> {
        let b = self.read_bytes(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }
}

/// The command entry point for decoding `.abi.bin` files back to JSON.
pub fn run(path_str: &str) -> Result<(), String> {
    let path = Path::new(path_str);
    if !path.exists() {
        return Err(format!("File not found: {}", path_str));
    }

    let data = fs::read(path).map_err(|e| format!("Read error: {}", e))?;
    let blob = decode_blob(&data).map_err(|e| format!("Decode error: {}", e))?;
    let json = blob_to_json(&blob);
    let pretty = pretty_json(&json);

    let out_path = output_path(path);
    fs::write(&out_path, &pretty).map_err(|e| format!("Write error: {}", e))?;

    tracing::info!("Decoded: {} -> {}", path.display(), out_path.display());

    Ok(())
}

fn output_path(src: &Path) -> std::path::PathBuf {
    let stem = src.file_stem().unwrap_or_default().to_string_lossy();
    let base = if stem.ends_with(".abi") {
        &stem[..stem.len() - 4]
    } else {
        &stem
    };
    src.with_file_name(format!("{}.json", base))
}

fn pretty_json(json: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(json) {
        if let Ok(s) = serde_json::to_string_pretty(&v) {
            return s;
        }
    }
    json.to_string()
}

pub fn decode_type(cur: &mut Cursor) -> Result<AbiType, &'static str> {
    let tag = cur.read_byte()?;
    match tag {
        T_UINT256 => Ok(AbiType::Uint256),
        T_UINT8   => Ok(AbiType::Uint8),
        T_UINT16  => Ok(AbiType::Uint16),
        T_UINT32  => Ok(AbiType::Uint32),
        T_UINT64  => Ok(AbiType::Uint64),
        T_UINT128 => Ok(AbiType::Uint128),
        T_ADDRESS => Ok(AbiType::Address),
        T_BOOL    => Ok(AbiType::Bool),
        T_BYTES32 => Ok(AbiType::Bytes32),
        T_BYTES_N => {
            let n = cur.read_byte()?;
            Ok(AbiType::BytesN(n))
        }
        T_STRING  => Ok(AbiType::AbiString),
        T_BYTES   => Ok(AbiType::Bytes),
        MOD_ARRAY => {
            let inner = decode_type(cur)?;
            Ok(AbiType::Array(Box::new(inner)))
        }
        MOD_TUPLE => {
            let count = cur.read_byte()? as usize;
            let mut fields = Vec::with_capacity(count);
            for _ in 0..count {
                fields.push(decode_type(cur)?);
            }
            Ok(AbiType::Tuple(fields))
        }
        _ => Err("unknown type tag"),
    }
}

fn decode_name(cur: &mut Cursor) -> Result<Vec<u8>, &'static str> {
    let len = cur.read_byte()? as usize;
    Ok(cur.read_bytes(len)?.to_vec())
}

fn decode_function(cur: &mut Cursor) -> Result<AbiFunction, &'static str> {
    let _selector = cur.read_bytes(4)?;
    let name = decode_name(cur)?;
    let input_count = cur.read_byte()? as usize;
    let mut inputs = Vec::with_capacity(input_count);
    for _ in 0..input_count {
        inputs.push(decode_type(cur)?);
    }
    let output_count = cur.read_byte()? as usize;
    let mut outputs = Vec::with_capacity(output_count);
    for _ in 0..output_count {
        outputs.push(decode_type(cur)?);
    }
    Ok(AbiFunction { name, inputs, outputs })
}

fn decode_event(cur: &mut Cursor) -> Result<AbiEvent, &'static str> {
    let name = decode_name(cur)?;
    let param_count = cur.read_byte()? as usize;
    let mut params = Vec::with_capacity(param_count);
    for _ in 0..param_count {
        let flags = cur.read_byte()?;
        let indexed = (flags & 0x01) != 0;
        let ty = decode_type(cur)?;
        params.push(EventParam { ty, indexed });
    }
    Ok(AbiEvent { name, params })
}

fn decode_error(cur: &mut Cursor) -> Result<AbiError, &'static str> {
    let _selector = cur.read_bytes(4)?;
    let name = decode_name(cur)?;
    let count = cur.read_byte()? as usize;
    let mut params = Vec::with_capacity(count);
    for _ in 0..count {
        params.push(decode_type(cur)?);
    }
    Ok(AbiError { name, params })
}

pub fn decode_blob(data: &[u8]) -> Result<AbiBlob, &'static str> {
    let mut cur = Cursor::new(data);
    let entry_count = cur.read_byte()? as usize;
    let mut blob = AbiBlob {
        functions: Vec::new(),
        events: Vec::new(),
        errors: Vec::new(),
    };
    for _ in 0..entry_count {
        let entry_type = cur.read_byte()?;
        let length = cur.read_u16_be()? as usize;
        let entry_data = cur.read_bytes(length)?;
        let mut ec = Cursor::new(entry_data);
        match entry_type {
            ENTRY_FUNCTION => blob.functions.push(decode_function(&mut ec)?),
            ENTRY_EVENT    => blob.events.push(decode_event(&mut ec)?),
            ENTRY_ERROR    => blob.errors.push(decode_error(&mut ec)?),
            _              => return Err("unknown entry type"),
        }
    }
    Ok(blob)
}

fn type_to_json(ty: &AbiType) -> String {
    match ty {
        AbiType::Uint256   => r#"{"type":"uint256"}"#.into(),
        AbiType::Uint8     => r#"{"type":"uint8"}"#.into(),
        AbiType::Uint16    => r#"{"type":"uint16"}"#.into(),
        AbiType::Uint32    => r#"{"type":"uint32"}"#.into(),
        AbiType::Uint64    => r#"{"type":"uint64"}"#.into(),
        AbiType::Uint128   => r#"{"type":"uint128"}"#.into(),
        AbiType::Address   => r#"{"type":"address"}"#.into(),
        AbiType::Bool      => r#"{"type":"bool"}"#.into(),
        AbiType::Bytes32   => r#"{"type":"bytes32"}"#.into(),
        AbiType::BytesN(n) => format!(r#"{{"type":"bytes{}"}}"#, n),
        AbiType::AbiString => r#"{"type":"string"}"#.into(),
        AbiType::Bytes     => r#"{"type":"bytes"}"#.into(),
        AbiType::Array(inner) => {
            format!(r#"{{"type":"array","inner":{}}}"#, type_to_json(inner))
        }
        AbiType::Tuple(fields) => {
            let parts: Vec<String> = fields.iter().map(type_to_json).collect();
            format!(r#"{{"type":"tuple","components":[{}]}}"#, parts.join(","))
        }
    }
}

pub fn blob_to_json(blob: &AbiBlob) -> String {
    let mut parts: Vec<String> = Vec::new();

    for f in &blob.functions {
        let name = String::from_utf8_lossy(&f.name);
        let inputs: Vec<String> = f.inputs.iter().map(type_to_json).collect();
        let outputs: Vec<String> = f.outputs.iter().map(type_to_json).collect();
        parts.push(format!(
            r#"{{"kind":"function","name":"{}","inputs":[{}],"outputs":[{}]}}"#,
            name,
            inputs.join(","),
            outputs.join(",")
        ));
    }

    for e in &blob.events {
        let name = String::from_utf8_lossy(&e.name);
        let params: Vec<String> = e.params.iter().map(|p| {
            format!(
                r#"{{"indexed":{},"type":{}}}"#,
                p.indexed,
                type_to_json(&p.ty)
            )
        }).collect();
        parts.push(format!(
            r#"{{"kind":"event","name":"{}","params":[{}]}}"#,
            name,
            params.join(",")
        ));
    }

    for e in &blob.errors {
        let name = String::from_utf8_lossy(&e.name);
        let params: Vec<String> = e.params.iter().map(type_to_json).collect();
        parts.push(format!(
            r#"{{"kind":"error","name":"{}","params":[{}]}}"#,
            name,
            params.join(",")
        ));
    }

    format!("[{}]", parts.join(","))
}
