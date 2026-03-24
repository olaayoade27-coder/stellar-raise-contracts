//! Tests for refund_single() token transfer logic.

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

use crate::{CrowdfundContract, CrowdfundContractClient};

fn setup_env() -> (
    Env,
    CrowdfundContractClient<'static>,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract_id.address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    let creator = Address::generate(&env);
    token_admin_client.mint(&creator, &10_000_000);

    (env, client, creator, token_address, token_admin)
}

fn mint_to(env: &Env, token_address: &Address, to: &Address, amount: i128) {
    let admin_client = token::StellarAssetClient::new(env, token_address);
    admin_client.mint(to, &amount);
}

fn default_init(
    client: &CrowdfundContractClient,
    creator: &Address,
    token_address: &Address,
    deadline: u64,
) {
    let admin = creator.clone();
    client.initialize(
        &admin,
        creator,
        token_address,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &None,
        &None,
    );
}

/// @notice refund_single returns contributed tokens and clears the contributor balance.
#[test]
fn test_refund_single_transfers_to_contributor_and_clears_balance() {
    let (env, client, creator, token_address, _token_admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let alice = Address::generate(&env);
    mint_to(&env, &token_address, &alice, 200_000);
    client.contribute(&alice, &200_000);

    env.ledger().set_timestamp(deadline + 1);
    client.refund_single(&alice);

    let token_client = token::Client::new(&env, &token_address);
    assert_eq!(token_client.balance(&alice), 200_000);
    assert_eq!(client.contribution(&alice), 0);
    assert_eq!(client.total_raised(), 0);
}

/// @notice refund_single only affects the targeted contributor.
#[test]
fn test_refund_single_only_updates_target_contributor() {
    let (env, client, creator, token_address, _token_admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint_to(&env, &token_address, &alice, 300_000);
    mint_to(&env, &token_address, &bob, 400_000);
    client.contribute(&alice, &300_000);
    client.contribute(&bob, &400_000);

    env.ledger().set_timestamp(deadline + 1);
    client.refund_single(&alice);

    assert_eq!(client.contribution(&alice), 0);
    assert_eq!(client.contribution(&bob), 400_000);
    assert_eq!(client.total_raised(), 400_000);
}

/// @notice refund_single before deadline returns CampaignStillActive.
#[test]
fn test_refund_single_before_deadline_returns_error() {
    let (env, client, creator, token_address, _token_admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let alice = Address::generate(&env);
    mint_to(&env, &token_address, &alice, 100_000);
    client.contribute(&alice, &100_000);

    let result = client.try_refund_single(&alice);
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::CampaignStillActive
    );
}

/// @notice refund_single when goal is reached returns GoalReached.
#[test]
fn test_refund_single_when_goal_reached_returns_error() {
    let (env, client, creator, token_address, _token_admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let alice = Address::generate(&env);
    mint_to(&env, &token_address, &alice, 1_000_000);
    client.contribute(&alice, &1_000_000);

    env.ledger().set_timestamp(deadline + 1);
    let result = client.try_refund_single(&alice);
    assert_eq!(result.unwrap_err().unwrap(), crate::ContractError::GoalReached);
}
