use serde::Deserialize;
use super::encode::{AbiBlob, AbiError, AbiEvent, AbiFunction, AbiType, EventParam};

#[derive(Debug, Deserialize)]
struct RawItem {
    #[serde(rename = "type", default = "default_type")]
    kind: String,
    name: Option<String>,
    #[serde(default)]
    inputs: Vec<RawParam>,
    #[serde(default)]
    outputs: Vec<RawParam>,
}

#[derive(Debug, Deserialize, Clone)]
struct RawParam {
    #[serde(rename = "type")]
    ty: String,
    #[serde(default)]
    indexed: bool,
    #[serde(default)]
    components: Vec<RawParam>,
    #[serde(default)]
    inner: Option<Box<RawParam>>,
}

fn default_type() -> String {
    "function".to_string()
}

fn parse_abi_type(param: &RawParam) -> AbiType {
    let ty = param.ty.trim();

    if ty.ends_with(']') {
        if let Some(bracket) = ty.rfind('[') {
            let base_ty = &ty[..bracket];
            let inner_param = RawParam {
                ty: base_ty.to_string(),
                indexed: false,
                components: param.components.clone(),
                inner: None,
            };
            return AbiType::Array(Box::new(parse_abi_type(&inner_param)));
        }
    }

    match ty {
        "address" => AbiType::Address,
        "bool"    => AbiType::Bool,
        "string"  => AbiType::AbiString,
        "bytes"   => AbiType::Bytes,
        "tuple"   => {
            let fields = param.components.iter().map(parse_abi_type).collect();
            AbiType::Tuple(fields)
        }
        "array"   => {
            if let Some(inner) = &param.inner {
                AbiType::Array(Box::new(parse_abi_type(inner)))
            } else {
                tracing::warn!("Array type missing 'inner' field, falling back to bytes");
                AbiType::Bytes
            }
        }
        "uint8"   => AbiType::Uint8,
        "uint16"  => AbiType::Uint16,
        "uint32"  => AbiType::Uint32,
        "uint64"  => AbiType::Uint64,
        "uint128" => AbiType::Uint128,
        "uint256" => AbiType::Uint256,
        "bytes1"  => AbiType::BytesN(1),
        "bytes2"  => AbiType::BytesN(2),
        "bytes3"  => AbiType::BytesN(3),
        "bytes4"  => AbiType::BytesN(4),
        s if s.starts_with("uint") || s.starts_with("int") => AbiType::Uint256,
        s if s.starts_with("bytes") && s.len() > 5 => {
            if let Ok(n) = s[5..].parse::<u8>() {
                if n == 32 {
                    AbiType::Bytes32
                } else if n >= 1 && n <= 4 {
                    AbiType::BytesN(n)
                } else {
                    AbiType::Bytes32 // Fallback for other bytesN
                }
            } else {
                AbiType::Bytes32
            }
        }
        _ => {
            tracing::warn!("Unknown ABI type '{}', falling back to bytes", ty);
            AbiType::Bytes
        }
    }
}

/// Parses a JSON string (either a bare ABI array or an object with an "abi" key) into an AbiBlob.
pub fn parse_abi_json(json: &str) -> Result<AbiBlob, String> {
    let root: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let array = match &root {
        serde_json::Value::Array(_) => root.clone(),
        serde_json::Value::Object(map) => {
            map.get("abi")
                .cloned()
                .ok_or_else(|| "JSON object has no \"abi\" field".to_string())?
        }
        _ => return Err("expected a JSON array or an object with an \"abi\" field".to_string()),
    };

    let items: Vec<RawItem> = serde_json::from_value(array)
        .map_err(|e| format!("ABI parse error: {}", e))?;

    let mut blob = AbiBlob {
        functions: Vec::new(),
        events:    Vec::new(),
        errors:    Vec::new(),
    };

    for item in &items {
        let name = match &item.name {
            Some(n) if !n.is_empty() => n.as_bytes().to_vec(),
            _ => continue,
        };

        match item.kind.as_str() {
            "function" => {
                blob.functions.push(AbiFunction {
                    name,
                    inputs:  item.inputs.iter().map(parse_abi_type).collect(),
                    outputs: item.outputs.iter().map(parse_abi_type).collect(),
                });
            }
            "event" => {
                blob.events.push(AbiEvent {
                    name,
                    params: item.inputs.iter().map(|p| EventParam {
                        ty:      parse_abi_type(p),
                        indexed: p.indexed,
                    }).collect(),
                });
            }
            "error" => {
                blob.errors.push(AbiError {
                    name,
                    params: item.inputs.iter().map(parse_abi_type).collect(),
                });
            }
            _ => {}
        }
    }

    Ok(blob)
}
