use crate::utils::names::ContractNames;

pub fn render(n: &ContractNames) -> String {
    let name  = &n.pascal;
    let camel = &n.camel;
    let lower = &n.lower;
    format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

/// @notice V2 storage struct — add new fields here, never touch V1 Structs.sol.
struct ModularStorageV2 {{
    uint256 newParam;
    // Add more V2 fields below
}}

library Lib{name}V2Storage {{
    bytes32 internal constant {camel}V2Point = keccak256("{lower}.storage.v2");

    function s() internal pure returns (ModularStorageV2 storage $) {{
        bytes32 slot = {camel}V2Point;
        assembly {{
            $.slot := slot
        }}
    }}
}}
"#
    )
}
