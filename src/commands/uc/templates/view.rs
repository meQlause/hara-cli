use crate::utils::names::ContractNames;

pub fn render(n: &ContractNames) -> String {
    let name = &n.pascal;
    format!(
        r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.29;

import {{Lib{name}Storage}} from "./{name}Storage.sol";

abstract contract {name}View {{
    using Lib{name}Storage for *;

    // Add view functions here
    // function getSomething(uint256 id) external view returns(uint256) {{
    //     return Lib{name}Storage.s().someField[id];
    // }}
}}
"#
    )
}
