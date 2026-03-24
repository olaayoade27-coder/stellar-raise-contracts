use soroban_sdk::{token, Address};

/// @title Refund Single Token Transfer Helper
/// @notice Centralizes transfer direction for contributor refunds.
/// @dev Keeps contract-side call sites explicit and typo-resistant for scripts.
pub fn refund_single_transfer(
    token_client: &token::Client,
    contract_address: &Address,
    contributor: &Address,
    amount: i128,
) {
    token_client.transfer(contract_address, contributor, &amount);
}
