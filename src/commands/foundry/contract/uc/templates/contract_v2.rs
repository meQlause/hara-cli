use crate::utils::names::ContractNames;

pub fn render(n: &ContractNames) -> String {
    let name = &n.pascal;
    format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import "./{name}.sol";
import "./libraries/{name}V2Storage.sol";

/// @title {name}V2
/// @notice Upgrade of {name} — adds V2 state via a separate Diamond Storage slot.
/// @dev Deploy with `hara uc` then run `script/Upgrade{name}.s.sol` to upgrade the proxy.
contract {name}V2 is {name} {{
    using Lib{name}V2Storage for *;

    /// @notice Re-initializer for V2 — called once during the upgrade transaction.
    /// @param  _newParam Example new parameter introduced in V2.
    function initializeV2(uint256 _newParam) public reinitializer(2) {{
        Lib{name}V2Storage.s().newParam = _newParam;
    }}

    // ─── V2 functions ──────────────────────────────────────────────────────────

    function getNewParam() external view returns (uint256) {{
        return Lib{name}V2Storage.s().newParam;
    }}
}}
"#
    )
}
