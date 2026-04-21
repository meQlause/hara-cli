use alloy::{
    network::TransactionBuilder,
    primitives::{keccak256, Address, Bytes, U256},
    providers::Provider,
    rpc::types::TransactionRequest,
    sol,
    sol_types::SolCall,
};
use eyre::{Result, Context};
use super::utils;

sol! {
    function setSubnodeOwner(bytes32 node, bytes32 label, address owner) external;
    function setAddr(bytes32 node, address a) external;
    function setAbi(bytes32 node, uint256 contentType, bytes data) external;
}

/// Content type constants as defined in EIP-205.
pub mod content_type {
    /// JSON-encoded ABI (contentType = 1).
    pub const JSON: u64 = 1;
    /// Raw binary bytecode (contentType = 8).
    pub const BIN: u64 = 8;
}

/// A client for interacting with the HARA Network Solution (HNS) registry.
pub struct HnsRegistry<P: Provider> {
    provider: P,
    proxy: Address,
    owner: Address,
}

impl<P: Provider> HnsRegistry<P> {
    /// Creates a new HNS registry client.
    pub fn new(provider: P, proxy: Address, owner: Address) -> Self {
        Self { provider, proxy, owner }
    }

    /// Registers a contract and its ABI/bytecode under a specific HARA subnode.
    pub async fn register_contract(
        &self,
        label: &str,
        contract_addr: Address,
        abi_data: Vec<u8>,
        content_type: u64,
    ) -> Result<()> {
        let node       = utils::calc_node(label);
        let parent     = utils::calc_parent_node();
        let label_hash = keccak256(label);

        tracing::info!("Registering '{}' (Node: {})", label, node);

        self.call_and_log(
            "setSubnodeOwner",
            setSubnodeOwnerCall { node: parent, label: label_hash, owner: self.owner }.abi_encode(),
        ).await?;

        self.call_and_log(
            "setAddr",
            setAddrCall { node, a: contract_addr }.abi_encode(),
        ).await?;

        self.call_and_log(
            "setAbi",
            setAbiCall {
                node,
                contentType: U256::from(content_type),
                data: Bytes::from(abi_data),
            }.abi_encode(),
        ).await?;

        tracing::info!("Successfully registered {}.hara.ethnet => {}", label, contract_addr);
        Ok(())
    }

    /// Executes a contract call, waits for the receipt, and logs the transaction status and fee.
    async fn call_and_log(&self, name: &str, input: Vec<u8>) -> Result<()> {
        let tx = TransactionRequest::default()
            .with_to(self.proxy)
            .with_input(input)
            .with_gas_price(0);

        let pending = self.provider.send_transaction(tx).await
            .wrap_err_with(|| format!("Failed to send {} transaction", name))?;

        let hash = pending.tx_hash();
        tracing::info!("{} transaction pending: {}", name, hash);

        let receipt = pending.get_receipt().await
            .wrap_err_with(|| format!("Failed to get receipt for {}", name))?;

        let gas   = receipt.gas_used;
        let price = receipt.effective_gas_price;
        let fee   = (gas as u128) * price;
        let fee_hara = alloy::primitives::utils::format_ether(U256::from(fee));

        tracing::info!(
            "{} transaction confirmed (Gas used: {}, Fee: {} HARA)",
            name, gas, fee_hara
        );

        Ok(())
    }
}
