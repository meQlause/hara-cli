/// Comprehensive ABI codec test suite.
///
/// Covers: round-trip integrity, recursive/nested tuples, arrays of tuples,
/// complex event params, multi-output functions, homogeneous arrays, deeply
/// nested structures, empty blobs, single-entry blobs, and JSON output structure.
#[cfg(test)]
mod tests {
    use super::super::encode::{
        AbiBlob, AbiError, AbiEvent, AbiFunction, AbiType, EventParam, encode_blob,
    };
    use super::super::decode::{decode_blob, blob_to_json};

    /// Encode → decode → re-encode and assert byte-level equality.
    fn assert_round_trip(blob: &AbiBlob) -> AbiBlob {
        let encoded    = encode_blob(blob);
        let decoded    = decode_blob(&encoded).expect("decode_blob failed");
        let re_encoded = encode_blob(&decoded);
        assert_eq!(
            encoded, re_encoded,
            "round-trip mismatch: original={} bytes  re-encoded={} bytes",
            encoded.len(), re_encoded.len()
        );
        decoded
    }

    #[test]
    fn test_empty_blob() {
        let blob = AbiBlob { functions: vec![], events: vec![], errors: vec![] };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_all_primitives() {
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"allPrimitives".to_vec(),
                inputs: vec![
                    AbiType::Uint256, AbiType::Address, AbiType::Bool,
                    AbiType::Bytes32, AbiType::AbiString, AbiType::Bytes,
                ],
                outputs: vec![AbiType::Bool, AbiType::Uint256],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_flat_tuple_input() {
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"deposit".to_vec(),
                inputs: vec![AbiType::Tuple(vec![AbiType::Address, AbiType::Uint256])],
                outputs: vec![AbiType::Bool],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_nested_tuple() {
        let meta  = AbiType::Tuple(vec![AbiType::Bool, AbiType::Bytes32]);
        let inner = AbiType::Tuple(vec![AbiType::Uint256, meta]);
        let outer = AbiType::Tuple(vec![AbiType::Address, inner]);
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"nestedStruct".to_vec(),
                inputs: vec![outer],
                outputs: vec![AbiType::Bool],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_deeply_nested_tuple() {
        let t1 = AbiType::Tuple(vec![AbiType::Uint256]);
        let t2 = AbiType::Tuple(vec![t1]);
        let t3 = AbiType::Tuple(vec![t2]);
        let t4 = AbiType::Tuple(vec![t3]);
        let t5 = AbiType::Tuple(vec![t4]);
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"deep".to_vec(),
                inputs: vec![t5],
                outputs: vec![],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_array_of_primitives() {
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"multiArray".to_vec(),
                inputs: vec![
                    AbiType::Array(Box::new(AbiType::Uint256)),
                    AbiType::Array(Box::new(AbiType::Address)),
                    AbiType::Array(Box::new(AbiType::Bytes32)),
                ],
                outputs: vec![AbiType::Array(Box::new(AbiType::Bool))],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_array_of_tuples() {
        let transfer = AbiType::Tuple(vec![AbiType::Address, AbiType::Uint256]);
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"batchTransfer".to_vec(),
                inputs: vec![AbiType::Array(Box::new(transfer))],
                outputs: vec![AbiType::Bool],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_nested_array() {
        let inner = AbiType::Array(Box::new(AbiType::Uint256));
        let outer = AbiType::Array(Box::new(inner));
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"matrix".to_vec(),
                inputs: vec![outer],
                outputs: vec![],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_array_of_array_of_tuples() {
        let point = AbiType::Tuple(vec![AbiType::Uint256, AbiType::Uint256]);
        let row   = AbiType::Array(Box::new(point));
        let grid  = AbiType::Array(Box::new(row));
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"grid".to_vec(),
                inputs: vec![grid],
                outputs: vec![],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_event_mixed_indexed() {
        let blob = AbiBlob {
            functions: vec![],
            events: vec![AbiEvent {
                name: b"Transfer".to_vec(),
                params: vec![
                    EventParam { ty: AbiType::Address, indexed: true  },
                    EventParam { ty: AbiType::Address, indexed: true  },
                    EventParam { ty: AbiType::Uint256, indexed: false },
                ],
            }],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_event_with_tuple_param() {
        let order_struct = AbiType::Tuple(vec![
            AbiType::Address, AbiType::Uint256, AbiType::Bytes32,
        ]);
        let blob = AbiBlob {
            functions: vec![],
            events: vec![AbiEvent {
                name: b"OrderFilled".to_vec(),
                params: vec![
                    EventParam { ty: order_struct,     indexed: true  },
                    EventParam { ty: AbiType::Address, indexed: false },
                ],
            }],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_event_with_nested_tuple() {
        let amounts = AbiType::Tuple(vec![AbiType::Uint256, AbiType::Uint256]);
        let pair    = AbiType::Tuple(vec![AbiType::Address, amounts]);
        let blob = AbiBlob {
            functions: vec![],
            events: vec![AbiEvent {
                name: b"Swap".to_vec(),
                params: vec![
                    EventParam { ty: pair,          indexed: true  },
                    EventParam { ty: AbiType::Bool, indexed: false },
                ],
            }],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_event_with_array_of_tuples() {
        let call_item = AbiType::Tuple(vec![AbiType::Address, AbiType::Bytes32]);
        let calls     = AbiType::Array(Box::new(call_item));
        let blob = AbiBlob {
            functions: vec![],
            events: vec![AbiEvent {
                name: b"BatchExecuted".to_vec(),
                params: vec![
                    EventParam { ty: calls,         indexed: false },
                    EventParam { ty: AbiType::Bool, indexed: false },
                ],
            }],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_error_with_nested_tuple() {
        let order = AbiType::Tuple(vec![AbiType::Address, AbiType::Uint256]);
        let blob = AbiBlob {
            functions: vec![],
            events:    vec![],
            errors: vec![AbiError {
                name: b"InvalidOrder".to_vec(),
                params: vec![order, AbiType::AbiString],
            }],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_multi_output_function() {
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"getState".to_vec(),
                inputs: vec![],
                outputs: vec![
                    AbiType::Address, AbiType::Uint256,
                    AbiType::Bool,    AbiType::Bytes32,
                ],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_function_returns_tuple() {
        let order = AbiType::Tuple(vec![AbiType::Address, AbiType::Uint256]);
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"getOrder".to_vec(),
                inputs: vec![AbiType::Bytes32],
                outputs: vec![order],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_function_returns_array_of_nested_tuples() {
        let pos_detail = AbiType::Tuple(vec![AbiType::Uint256, AbiType::Bool]);
        let position   = AbiType::Tuple(vec![AbiType::Address, pos_detail]);
        let positions  = AbiType::Array(Box::new(position));
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"getPositions".to_vec(),
                inputs: vec![],
                outputs: vec![positions],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_tuple_containing_array() {
        let ids   = AbiType::Array(Box::new(AbiType::Uint256));
        let claim = AbiType::Tuple(vec![AbiType::Address, ids, AbiType::Bytes32]);
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"claim".to_vec(),
                inputs: vec![claim],
                outputs: vec![AbiType::Bool],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_tuple_containing_array_of_tuples() {
        let hop   = AbiType::Tuple(vec![AbiType::Address, AbiType::Uint256]);
        let hops  = AbiType::Array(Box::new(hop));
        let route = AbiType::Tuple(vec![AbiType::Address, hops, AbiType::Uint256]);
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"swap".to_vec(),
                inputs: vec![route],
                outputs: vec![AbiType::Uint256],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_full_complex_blob() {
        let hop   = AbiType::Tuple(vec![AbiType::Address, AbiType::Uint256]);
        let hops  = AbiType::Array(Box::new(hop));
        let route = AbiType::Tuple(vec![AbiType::Address, hops, AbiType::Uint256]);
        let call  = AbiType::Tuple(vec![AbiType::Address, AbiType::Bytes32]);
        let calls = AbiType::Array(Box::new(call));
        let meta  = AbiType::Tuple(vec![AbiType::Address, AbiType::Uint256]);
        let pair  = AbiType::Tuple(vec![AbiType::Address, AbiType::Address]);

        let blob = AbiBlob {
            functions: vec![
                AbiFunction {
                    name: b"execute".to_vec(),
                    inputs: vec![route, calls],
                    outputs: vec![AbiType::Bool, AbiType::Uint256],
                },
                AbiFunction {
                    name: b"pause".to_vec(),
                    inputs: vec![],
                    outputs: vec![],
                },
            ],
            events: vec![
                AbiEvent {
                    name: b"Executed".to_vec(),
                    params: vec![
                        EventParam { ty: meta,             indexed: true  },
                        EventParam { ty: AbiType::Bool,    indexed: false },
                    ],
                },
                AbiEvent {
                    name: b"Transfer".to_vec(),
                    params: vec![
                        EventParam { ty: AbiType::Address, indexed: true  },
                        EventParam { ty: AbiType::Address, indexed: true  },
                        EventParam { ty: AbiType::Uint256, indexed: false },
                    ],
                },
            ],
            errors: vec![
                AbiError {
                    name: b"SlippageExceeded".to_vec(),
                    params: vec![pair, AbiType::Uint256, AbiType::Uint256],
                },
                AbiError {
                    name: b"Paused".to_vec(),
                    params: vec![],
                },
            ],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_blob_to_json_structure() {
        let inner = AbiType::Tuple(vec![AbiType::Address, AbiType::Uint256]);
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"fn".to_vec(),
                inputs: vec![inner],
                outputs: vec![AbiType::Bool],
            }],
            events: vec![],
            errors: vec![],
        };
        let encoded = encode_blob(&blob);
        let decoded = decode_blob(&encoded).expect("decode failed");
        let json    = blob_to_json(&decoded);

        let open  = json.chars().filter(|&c| c == '[' || c == '{').count();
        let close = json.chars().filter(|&c| c == ']' || c == '}').count();
        assert_eq!(open, close, "JSON has unbalanced brackets:\n{}", json);
        assert!(json.starts_with('['), "JSON must start with '['");
        assert!(json.ends_with(']'),   "JSON must end with ']'");
    }

    #[test]
    fn test_no_input_no_output_function() {
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"renounceOwnership".to_vec(),
                inputs: vec![],
                outputs: vec![],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_large_flat_tuple() {
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"bigStruct".to_vec(),
                inputs: vec![AbiType::Tuple(vec![
                    AbiType::Address, AbiType::Uint256, AbiType::Bool,    AbiType::Bytes32,
                    AbiType::AbiString, AbiType::Bytes, AbiType::Address, AbiType::Uint256,
                    AbiType::Bool,    AbiType::Bytes32,
                ])],
                outputs: vec![],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    // ─── Additional tests ───────────────────────────────────────────────────

    #[test]
    fn test_single_event_no_params() {
        let blob = AbiBlob {
            functions: vec![],
            events: vec![AbiEvent { name: b"Pause".to_vec(), params: vec![] }],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_single_error_no_params() {
        let blob = AbiBlob {
            functions: vec![],
            events:    vec![],
            errors:    vec![AbiError { name: b"Unauthorized".to_vec(), params: vec![] }],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_multiple_functions() {
        let blob = AbiBlob {
            functions: vec![
                AbiFunction { name: b"mint".to_vec(),     inputs: vec![AbiType::Address, AbiType::Uint256], outputs: vec![] },
                AbiFunction { name: b"burn".to_vec(),     inputs: vec![AbiType::Uint256], outputs: vec![] },
                AbiFunction { name: b"transfer".to_vec(), inputs: vec![AbiType::Address, AbiType::Uint256], outputs: vec![AbiType::Bool] },
                AbiFunction { name: b"approve".to_vec(),  inputs: vec![AbiType::Address, AbiType::Uint256], outputs: vec![AbiType::Bool] },
                AbiFunction { name: b"balanceOf".to_vec(),inputs: vec![AbiType::Address], outputs: vec![AbiType::Uint256] },
            ],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_string_and_bytes_fields_in_tuple() {
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"storeMetadata".to_vec(),
                inputs: vec![AbiType::Tuple(vec![
                    AbiType::AbiString,
                    AbiType::Bytes,
                    AbiType::Uint256,
                ])],
                outputs: vec![AbiType::Bool],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_triple_nested_array() {
        // uint256[][][]
        let t1 = AbiType::Array(Box::new(AbiType::Uint256));
        let t2 = AbiType::Array(Box::new(t1));
        let t3 = AbiType::Array(Box::new(t2));
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"cube".to_vec(),
                inputs: vec![t3],
                outputs: vec![],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }

    #[test]
    fn test_blob_to_json_contains_function_name() {
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"mySpecialFn".to_vec(),
                inputs: vec![AbiType::Address],
                outputs: vec![AbiType::Bool],
            }],
            events: vec![],
            errors: vec![],
        };
        let encoded = encode_blob(&blob);
        let decoded = decode_blob(&encoded).expect("decode failed");
        let json    = blob_to_json(&decoded);
        assert!(json.contains("mySpecialFn"), "JSON should contain the function name");
        assert!(json.contains("address"),     "JSON should contain input type");
        assert!(json.contains("bool"),        "JSON should contain output type");
    }

    #[test]
    fn test_blob_to_json_event_name() {
        let blob = AbiBlob {
            functions: vec![],
            events: vec![AbiEvent {
                name: b"UniqueEventXyz".to_vec(),
                params: vec![EventParam { ty: AbiType::Uint256, indexed: true }],
            }],
            errors: vec![],
        };
        let encoded = encode_blob(&blob);
        let decoded = decode_blob(&encoded).expect("decode failed");
        let json    = blob_to_json(&decoded);
        assert!(json.contains("UniqueEventXyz"), "JSON should contain event name");
        assert!(json.contains("true"),           "JSON should contain indexed:true");
    }

    #[test]
    fn test_expanded_primitives() {
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name: b"expanded".to_vec(),
                inputs: vec![
                    AbiType::Uint8, AbiType::Uint16, AbiType::Uint32,
                    AbiType::Uint64, AbiType::Uint128,
                    AbiType::BytesN(1), AbiType::BytesN(2),
                    AbiType::BytesN(3), AbiType::BytesN(4),
                ],
                outputs: vec![],
            }],
            events: vec![],
            errors: vec![],
        };
        assert_round_trip(&blob);
    }
}

/// Failure and negative test cases for the ABI codec.
#[cfg(test)]
mod tests_fail {
    use super::super::decode::decode_blob;
    use super::super::encode::{AbiBlob, AbiFunction, AbiType, encode_blob};
    use super::super::parser::parse_abi_json;

    #[test]
    fn fail_parse_empty_string() {
        assert!(parse_abi_json("").is_err(), "empty string should fail");
    }

    #[test]
    fn fail_parse_json_object_not_array() {
        assert!(
            parse_abi_json(r#"{"type":"function"}"#).is_err(),
            "object at root should fail"
        );
    }

    #[test]
    fn fail_parse_plain_string() {
        assert!(parse_abi_json(r#""hello""#).is_err(), "a bare string should fail");
    }

    #[test]
    fn fail_parse_truncated_json() {
        assert!(
            parse_abi_json(r#"[{"type":"function","name":"foo","inputs":["#).is_err(),
            "truncated JSON should fail"
        );
    }

    #[test]
    fn fail_parse_inputs_wrong_type() {
        let json = r#"[{"type":"function","name":"foo","inputs":"bad"}]"#;
        assert!(parse_abi_json(json).is_err(), "inputs as a string should fail");
    }

    #[test]
    fn fail_parse_null_type_field() {
        let json = r#"[{"type":null,"name":"foo","inputs":[]}]"#;
        assert!(parse_abi_json(json).is_err(), "null type field should fail");
    }

    #[test]
    fn fail_decode_empty_bytes() {
        assert!(decode_blob(&[]).is_err(), "empty bytes should fail");
    }

    #[test]
    fn fail_decode_count_without_entries() {
        assert!(decode_blob(&[0x01]).is_err(), "entry count with no data should fail");
    }

    #[test]
    fn fail_decode_missing_length_field() {
        assert!(decode_blob(&[0x01, 0x01]).is_err(), "missing length field should fail");
    }

    #[test]
    fn fail_decode_unknown_entry_type() {
        assert!(
            decode_blob(&[0x01, 0xFF, 0x00, 0x01, 0x00]).is_err(),
            "unknown entry type should fail"
        );
    }

    #[test]
    fn fail_decode_truncated_entry_payload() {
        assert!(
            decode_blob(&[0x01, 0x01, 0x00, 0x04, 0xAA, 0xBB]).is_err(),
            "truncated entry payload should fail"
        );
    }

    #[test]
    fn fail_decode_unknown_type_tag_inside_entry() {
        let blob = AbiBlob {
            functions: vec![AbiFunction {
                name:    b"foo".to_vec(),
                inputs:  vec![AbiType::Bool],
                outputs: vec![],
            }],
            events: vec![],
            errors: vec![],
        };
        let mut encoded = encode_blob(&blob);
        if encoded.len() > 13 {
            encoded[13] = 0xEE;
        }
        assert!(decode_blob(&encoded).is_err(), "unknown type tag in payload should fail");
    }
}
