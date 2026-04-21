use crate::utils::names::ContractNames;

pub fn render(n: &ContractNames) -> String {
    let name = &n.pascal;
    format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import "forge-std/Script.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "../src/{name}.sol";

contract Deploy{name}Script is Script {{
    function run() external {{
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);

        vm.startBroadcast(deployerPrivateKey);

        {name} implementation = new {name}();

        bytes memory initData = abi.encodeWithSelector(
            {name}.initialize.selector,
            deployer
        );

        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            initData
        );

        vm.stopBroadcast();
    }}
}}
"#
    )
}
