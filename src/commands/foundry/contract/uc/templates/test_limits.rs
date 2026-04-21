use crate::utils::names::ContractNames;

pub fn render(n: &ContractNames) -> String {
    let name = &n.pascal;
    format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import {{Test, console}} from "forge-std/Test.sol";
import {{ {name} }} from "../src/{name}.sol";
import {{ERC1967Proxy}} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

/// @title ContractLimitsTest
/// @notice CI enforcement suite — all tests here MUST pass on every push / PR.
/// @dev  EIP-170 hard limit : 24,576 bytes deployed bytecode per contract.
/// @dev  Gas soft limit     : 300,000 gas per public method call.
contract ContractLimitsTest is Test {{
    uint256 constant MAX_CONTRACT_SIZE = 24_576; // EIP-170 hard limit (bytes)
    uint256 constant MAX_GAS_PER_CALL  = 300_000; // project policy limit (gas units)

    {name}    public impl;
    {name}    public instance;
    ERC1967Proxy public proxy;

    address admin = makeAddr("admin");

    function setUp() public {{
        impl = new {name}();

        bytes memory initData = abi.encodeWithSelector({name}.initialize.selector, admin);
        proxy    = new ERC1967Proxy(address(impl), initData);
        instance = {name}(address(proxy));
    }}

    // ═══════════════════════════════════════════════════════
    // CONTRACT SIZE CHECKS  (must be < 24,576 bytes — EIP-170)
    // ═══════════════════════════════════════════════════════

    function test_size_{name}_implementation() public view {{
        uint256 size = address(impl).code.length;
        console.log("[SIZE] {name} (impl):", size, "/ 24576 bytes");
        assertLt(size, MAX_CONTRACT_SIZE, _sizeErr("{name} (impl)", size));
    }}

    function test_size_ERC1967Proxy() public view {{
        uint256 size = address(proxy).code.length;
        console.log("[SIZE] ERC1967Proxy:", size, "/ 24576 bytes");
        assertLt(size, MAX_CONTRACT_SIZE, _sizeErr("ERC1967Proxy", size));
    }}

    // ═══════════════════════════════════════════════════════
    // GAS LIMIT CHECKS  (must be < 300,000 gas per call)
    // ═══════════════════════════════════════════════════════

    function test_gas_initialize() public {{
        {name} newImpl = new {name}();
        bytes memory initData = abi.encodeWithSelector({name}.initialize.selector, admin);

        uint256 gasStart = gasleft();
        new ERC1967Proxy(address(newImpl), initData);
        uint256 gasUsed = gasStart - gasleft();

        console.log("[GAS]  initialize():", gasUsed, "/ 300000");
        assertLt(gasUsed, MAX_GAS_PER_CALL, _gasErr("initialize()", gasUsed));
    }}

    function test_gas_exampleAction_coldSlot() public {{
        uint256 gasStart = gasleft();
        instance.exampleAction(1);
        uint256 gasUsed = gasStart - gasleft();

        console.log("[GAS]  exampleAction() [cold slot]:", gasUsed, "/ 300000");
        assertLt(gasUsed, MAX_GAS_PER_CALL, _gasErr("exampleAction() [cold]", gasUsed));
    }}

    function test_gas_exampleAction_warmSlot() public {{
        instance.exampleAction(1); // warm the slot
        uint256 gasStart = gasleft();
        instance.exampleAction(1);
        uint256 gasUsed = gasStart - gasleft();

        console.log("[GAS]  exampleAction() [warm slot]:", gasUsed, "/ 300000");
        assertLt(gasUsed, MAX_GAS_PER_CALL, _gasErr("exampleAction() [warm]", gasUsed));
    }}

    // ─────────────────────────────────────────────────────────
    // Internal helpers
    // ─────────────────────────────────────────────────────────
    function _sizeErr(string memory name, uint256 size) internal pure returns (string memory) {{
        return string.concat(
            name, " exceeds 24 KB EVM limit: ", vm.toString(size), " bytes (max 24576)"
        );
    }}

    function _gasErr(string memory name, uint256 gas) internal pure returns (string memory) {{
        return string.concat(
            name, " exceeds 300K gas policy: ", vm.toString(gas), " gas (max 300000)"
        );
    }}
}}
"#
    )
}
