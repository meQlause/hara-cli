use crate::utils::names::ContractNames;

pub fn render(n: &ContractNames) -> String {
    let name = &n.pascal;
    format!(
        r#"name: Contract Limits (Size & Gas)

on:
  push:
  pull_request:
  workflow_dispatch:

jobs:
  contract-limits:
    name: Size < 24 KB · Gas < 300K
    runs-on: ubuntu-latest

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          submodules: recursive

      - name: Install Foundry
        uses: foundry-rs/foundry-toolchain@v1

      - name: Show Forge version
        run: forge --version

      - name: Build contracts
        run: forge build --silent

      - name: Check contract sizes (< 24,576 bytes — EIP-170)
        run: |
          echo "──────────────────────────────────────────────────────"
          echo "📏 Contract size report"
          echo "──────────────────────────────────────────────────────"

          SIZE_OUTPUT=$(forge build --sizes 2>&1)
          echo "$SIZE_OUTPUT"

          if echo "$SIZE_OUTPUT" | grep -qE '\|\s*-[0-9]'; then
            echo ""
            echo "::error::One or more contracts exceed the 24 KB EVM size limit (EIP-170)."
            exit 1
          fi

          echo ""
          echo "✅ All contracts are within the 24 KB EVM limit."

      - name: Run ContractLimitsTest ({name})
        run: |
          echo "──────────────────────────────────────────────────────"
          echo "⛽ Size & Gas suite — ContractLimitsTest"
          echo "──────────────────────────────────────────────────────"
          forge test --match-contract "ContractLimitsTest" -vvv

      - name: Run full test suite
        run: |
          echo "──────────────────────────────────────────────────────"
          echo "🧪 Full test suite"
          echo "──────────────────────────────────────────────────────"
          forge test -vvv

      - name: Generate gas snapshot
        run: forge snapshot --snap .gas-snapshot

      - name: Upload gas snapshot
        uses: actions/upload-artifact@v4
        with:
          name: gas-snapshot-{name}
          path: .gas-snapshot
          retention-days: 30
"#
    )
}
