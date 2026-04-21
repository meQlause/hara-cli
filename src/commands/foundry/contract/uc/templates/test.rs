use crate::utils::names::ContractNames;

pub fn render(n: &ContractNames) -> String {
    let name = &n.pascal;
    format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import "forge-std/Test.sol";
import {{ {name} }} from "../src/{name}.sol";
import {{ ERC1967Proxy }} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

contract {name}Test is Test {{
    {name} public contractInstance;
    address public owner = address(1);

    function setUp() public {{
        {name} implementation = new {name}();
        bytes memory data = abi.encodeWithSelector({name}.initialize.selector, owner);
        ERC1967Proxy proxy = new ERC1967Proxy(address(implementation), data);
        contractInstance = {name}(address(proxy));
    }}

    function test_Initialization() public {{
        assertEq(contractInstance.owner(), owner);
    }}
}}
"#
    )
}
