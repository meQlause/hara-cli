use crate::utils::names::ContractNames;

pub fn render(n: &ContractNames) -> String {
    let name  = &n.pascal;
    let camel = &n.camel;
    let lower = &n.lower;
    format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import {{ModularStorage}} from "./Structs.sol";

library Lib{name}Storage {{
    bytes32 internal constant {camel}Point = keccak256("{lower}.storage");

    function s() internal pure returns(ModularStorage storage $) {{
        bytes32 slot = {camel}Point;
        assembly {{
            $.slot := slot
        }}
    }}
}}
"#
    )
}
