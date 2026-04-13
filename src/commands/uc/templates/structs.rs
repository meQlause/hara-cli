pub fn render() -> String {
    r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.29;

struct ModularStorage {
    mapping(uint256 => uint256) idToCounter;
}
"#
    .to_string()
}
