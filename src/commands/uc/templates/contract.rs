use crate::utils::names::ContractNames;

pub fn render(n: &ContractNames) -> String {
    let name = &n.pascal;
    format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import {{Initializable}} from "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import {{OwnableUpgradeable}} from "@openzeppelin-upgradeable/contracts/access/OwnableUpgradeable.sol";
import {{ {name}View}} from "./libraries/{name}View.sol";
import {{Lib{name}Storage}} from "./libraries/{name}Storage.sol";

contract {name} is Initializable, OwnableUpgradeable, {name}View {{
    using Lib{name}Storage for *;

    function initialize(address adminOwner) public initializer {{
        __Ownable_init(adminOwner);
        // Additional initialization logic
    }}

    constructor() {{
        _disableInitializers();
    }}

    // Example function using Diamond Storage
    function exampleAction(uint256 id) public virtual {{
        // Lib{name}Storage.s().idToCounter[id]++;
    }}
}}
"#
    )
}
