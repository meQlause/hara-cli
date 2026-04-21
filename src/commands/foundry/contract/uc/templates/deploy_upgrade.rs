use crate::utils::names::ContractNames;

pub fn render(n: &ContractNames) -> String {
    let name = &n.pascal;
    format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import "forge-std/Script.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "../src/{name}V2.sol";

/// @title Upgrade{name}Script
/// @notice Upgrades an existing ERC-1967 proxy to {name}V2 and calls initializeV2.
///
/// Usage:
///   forge script script/Upgrade{name}.s.sol \
///     --rpc-url $RPC_URL \
///     --broadcast \
///     -vvvv
///
/// Environment variables required:
///   PRIVATE_KEY  — deployer private key
///   PROXY_ADDR   — address of the existing ERC-1967 proxy
contract Upgrade{name}Script is Script {{
    function run() external {{
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address proxyAddr          = vm.envAddress("PROXY_ADDR");

        vm.startBroadcast(deployerPrivateKey);

        // 1. Deploy the new implementation
        {name}V2 newImpl = new {name}V2();

        // 2. Encode the V2 initializer call
        bytes memory initData = abi.encodeWithSelector(
            {name}V2.initializeV2.selector,
            0   // <-- replace with your actual _newParam value
        );

        // 3. Upgrade the proxy to V2 and call initializeV2 atomically
        {name}V2(payable(proxyAddr)).upgradeToAndCall(
            address(newImpl),
            initData
        );

        vm.stopBroadcast();
    }}
}}
"#
    )
}
