#![no_std]
#![allow(clippy::too_many_arguments)]

use soroban_sdk::{
    contract, contractclient, contractimpl, contracttype, token, Address, Env, String, Symbol, Vec,
};

use crate::contribute_error_handling::log_contribute_error;


// ── Modules ───────────────────────────────────────────────────────────────────

pub mod admin_upgrade_mechanism;
pub mod campaign_goal_minimum;
pub mod cargo_toml_rust;
pub mod contract_state_size;
pub mod contribute_error_handling;
pub mod crowdfund_initialize_function;
pub mod proptest_generator_boundary;
pub mod refund_single_token;
pub mod soroban_sdk_minor;
pub mod stellar_token_minter;
pub mod withdraw_event_emission;

use crowdfund_initialize_function::{execute_initialize, InitParams};
use refund_single_token::{execute_refund_single, refund_single_transfer, validate_refund_preconditions};
use withdraw_event_emission::{emit_withdrawal_event, mint_nfts_in_batch};

// ── Test modules ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod auth_tests;
#[cfg(test)]
#[path = "admin_upgrade_mechanism.test.rs"]
mod admin_upgrade_mechanism_test;
#[cfg(test)]
#[path = "campaign_goal_minimum.test.rs"]
mod campaign_goal_minimum_test;
#[cfg(test)]
#[path = "cargo_toml_rust.test.rs"]
mod cargo_toml_rust_test;
#[cfg(test)]
#[path = "contract_state_size.test.rs"]
mod contract_state_size_test;
#[cfg(test)]
mod contribute_error_handling_tests;
#[cfg(test)]
#[path = "crowdfund_initialize_function.test.rs"]
mod crowdfund_initialize_function_test;
#[cfg(test)]
#[path = "proptest_generator_boundary.test.rs"]
mod proptest_generator_boundary_tests;
#[cfg(test)]
#[path = "refund_single_token.test.rs"]
mod refund_single_token_test;
#[cfg(test)]
mod stellar_token_minter_test;
#[cfg(test)]
mod test;
#[cfg(test)]
mod withdraw_event_emission_test;

// ── Constants ─────────────────────────────────────────────────────────────────

const CONTRACT_VERSION: u32 = 3;
#[allow(dead_code)]
const CONTRIBUTION_COOLDOWN: u64 = 60;

pub const MAX_NFT_MINT_BATCH: u32 = 50;

// ── Data Types ────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum Status {
    Active,
    Succeeded,
    Expired,
    Cancelled,
}

#[derive(Clone)]
#[contracttype]
pub struct RoadmapItem {
    pub date: u64,
    pub description: String,
}

#[derive(Clone)]
#[contracttype]
pub struct PlatformConfig {
    pub address: Address,
    pub fee_bps: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct CampaignStats {
    pub total_raised: i128,
    pub goal: i128,
    pub progress_bps: u32,
    pub contributor_count: u32,
    pub average_contribution: i128,
    pub largest_contribution: i128,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Creator,
    Token,
    Goal,
    Deadline,
    TotalRaised,
    Contribution(Address),
    Contributors,
    Status,
    MinContribution,
    Pledge(Address),
    TotalPledged,
    StretchGoals,
    BonusGoal,
    BonusGoalDescription,
    BonusGoalReachedEmitted,
    Pledgers,
    Roadmap,
    Admin,
    Title,
    Description,
    SocialLinks,
    PlatformConfig,
    NFTContract,
    TokenDecimals,
}

// ── Contract Error ────────────────────────────────────────────────────────────

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    CampaignEnded = 2,
    CampaignStillActive = 3,
    GoalNotReached = 4,
    GoalReached = 5,
    Overflow = 6,
    NothingToRefund = 7,
    /// `goal < MIN_GOAL_AMOUNT`
    InvalidGoal = 8,
    /// `min_contribution < MIN_CONTRIBUTION_AMOUNT`
    InvalidMinContribution = 9,
    /// `deadline < now + MIN_DEADLINE_OFFSET`
    DeadlineTooSoon = 10,
    /// `platform_config.fee_bps > MAX_PLATFORM_FEE_BPS`
    InvalidPlatformFee = 11,
    /// `bonus_goal <= goal`
    InvalidBonusGoal = 12,
    /// `amount == 0`
    ZeroAmount = 13,
    /// `amount < min_contribution`
    BelowMinimum = 14,
    /// Campaign is not in `Active` status
    CampaignNotActive = 15,
    /// `amount <= 0` or `amount < min_contribution`
    AmountTooLow = 16,
}

// ── NFT contract interface ────────────────────────────────────────────────────

#[contractclient(name = "NftContractClient")]
pub trait NftContract {
    fn mint(env: Env, to: Address) -> u128;
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct CrowdfundContract;

#[contractimpl]
impl CrowdfundContract {
    /// Initializes a new crowdfunding campaign.
    pub fn initialize(
        env: Env,
        admin: Address,
        creator: Address,
        token: Address,
        goal: i128,
        deadline: u64,
        min_contribution: i128,
        platform_config: Option<PlatformConfig>,
        bonus_goal: Option<i128>,
        bonus_goal_description: Option<String>,
        _metadata_uri: Option<String>,
    ) -> Result<(), ContractError> {
        execute_initialize(
            &env,
            InitParams {
                admin,
                creator,
                token,
                goal,
                deadline,
                min_contribution,
                platform_config,
                bonus_goal,
                bonus_goal_description,
            },
        )
    }

    /// Returns the list of all contributor addresses.
    pub fn contributors(env: Env) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or(Vec::new(&env))
    }

    /// Contribute tokens to the campaign.
    pub fn contribute(env: Env, contributor: Address, amount: i128) -> Result<(), ContractError> {
        contributor.require_auth();

        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            log_contribute_error(&env, ContractError::CampaignNotActive);
            return Err(ContractError::CampaignNotActive);
        }


        if amount == 0 {
            log_contribute_error(&env, ContractError::ZeroAmount);
            return Err(ContractError::ZeroAmount);
        } 
        if amount < 0 {
            log_contribute_error(&env, ContractError::AmountTooLow);
            return Err(ContractError::AmountTooLow);
        }


        let min_contribution: i128 = env
            .storage()
            .instance()
            .get(&DataKey::MinContribution)
            .unwrap();
        if amount < min_contribution {
            log_contribute_error(&env, ContractError::BelowMinimum);
            return Err(ContractError::BelowMinimum);
        }


        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        if env.ledger().timestamp() > deadline {
            log_contribute_error(&env, ContractError::CampaignEnded);
            return Err(ContractError::CampaignEnded);
        }

        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();

        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&contributor, &env.current_contract_address(), &amount);

        let contribution_key = DataKey::Contribution(contributor.clone());
        let previous_amount: i128 = env
            .storage()
            .persistent()
            .get(&contribution_key)
            .unwrap_or(0);
        let new_contribution = previous_amount
            .checked_add(amount)
            .ok_or(ContractError::Overflow)?;
        env.storage().persistent().set(&contribution_key, &new_contribution);
        env.storage().persistent().extend_ttl(&contribution_key, 100, 100);

        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();
        let new_total = total.checked_add(amount).ok_or(ContractError::Overflow)?;
        env.storage().instance().set(&DataKey::TotalRaised, &new_total);

        if let Some(bg) = env.storage().instance().get::<_, i128>(&DataKey::BonusGoal) {
            let already_emitted = env
                .storage()
                .instance()
                .get::<_, bool>(&DataKey::BonusGoalReachedEmitted)
                .unwrap_or(false);
            if !already_emitted && total < bg && new_total >= bg {
                env.events().publish(("campaign", "bonus_goal_reached"), bg);
                env.storage().instance().set(&DataKey::BonusGoalReachedEmitted, &true);
            }
        }

        let mut contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or_else(|| Vec::new(&env));
        if !contributors.contains(&contributor) {
            contributors.push_back(contributor.clone());
            env.storage().persistent().set(&DataKey::Contributors, &contributors);
            env.storage().persistent().extend_ttl(&DataKey::Contributors, 100, 100);
        }

        env.events().publish(("campaign", "contributed"), (contributor, amount));
        Ok(())
    }

    /// Sets the NFT contract address — only callable by the creator.
    pub fn set_nft_contract(env: Env, creator: Address, nft_contract: Address) {
        let stored_creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        if creator != stored_creator {
            panic!("not authorized");
        }
        creator.require_auth();
        env.storage().instance().set(&DataKey::NFTContract, &nft_contract);
    }

    /// Pledge tokens without transferring them immediately.
    pub fn pledge(env: Env, pledger: Address, amount: i128) -> Result<(), ContractError> {
        pledger.require_auth();

        let min_contribution: i128 = env
            .storage()
            .instance()
            .get(&DataKey::MinContribution)
            .unwrap();
        if amount < min_contribution {
            return Err(ContractError::BelowMinimum);
        }

        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        if env.ledger().timestamp() > deadline {
            return Err(ContractError::CampaignEnded);
        }

        let pledge_key = DataKey::Pledge(pledger.clone());
        let prev: i128 = env.storage().persistent().get(&pledge_key).unwrap_or(0);
        env.storage().persistent().set(&pledge_key, &(prev + amount));
        env.storage().persistent().extend_ttl(&pledge_key, 100, 100);

        let total_pledged: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalPledged)
            .unwrap_or(0);
        env.storage().instance().set(&DataKey::TotalPledged, &(total_pledged + amount));

        let mut pledgers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Pledgers)
            .unwrap_or_else(|| Vec::new(&env));
        if !pledgers.contains(&pledger) {
            pledgers.push_back(pledger.clone());
            env.storage().persistent().set(&DataKey::Pledgers, &pledgers);
            env.storage().persistent().extend_ttl(&DataKey::Pledgers, 100, 100);
        }

        env.events().publish(("campaign", "pledged"), (pledger, amount));
        Ok(())
    }

    /// Collect all pledges after the deadline when the goal is met.
    pub fn collect_pledges(env: Env) -> Result<(), ContractError> {
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            panic!("campaign is not active");
        }

        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        if env.ledger().timestamp() <= deadline {
            return Err(ContractError::CampaignStillActive);
        }

        let goal: i128 = env.storage().instance().get(&DataKey::Goal).unwrap();
        let total_raised: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();
        let total_pledged: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalPledged)
            .unwrap_or(0);

        if total_raised + total_pledged < goal {
            return Err(ContractError::GoalNotReached);
        }

        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_address);

        let pledgers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Pledgers)
            .unwrap_or_else(|| Vec::new(&env));

        for pledger in pledgers.iter() {
            let pledge_key = DataKey::Pledge(pledger.clone());
            let amount: i128 = env.storage().persistent().get(&pledge_key).unwrap_or(0);
            if amount > 0 {
                token_client.transfer(&pledger, &env.current_contract_address(), &amount);
                env.storage().persistent().set(&pledge_key, &0i128);
                env.storage().persistent().extend_ttl(&pledge_key, 100, 100);
            }
        }

        env.storage().instance().set(&DataKey::TotalRaised, &(total_raised + total_pledged));
        env.storage().instance().set(&DataKey::TotalPledged, &0i128);
        env.events().publish(("campaign", "pledges_collected"), total_pledged);
        Ok(())
    }

    /// Finalize the campaign: Active → Succeeded or Active → Expired.
    pub fn finalize(env: Env) -> Result<Status, ContractError> {
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            panic!("campaign is not active");
        }

        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        if env.ledger().timestamp() <= deadline {
            return Err(ContractError::CampaignStillActive);
        }

        let goal: i128 = env.storage().instance().get(&DataKey::Goal).unwrap();
        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap_or(0);

        let new_status = if total >= goal { Status::Succeeded } else { Status::Expired };
        env.storage().instance().set(&DataKey::Status, &new_status);
        env.events().publish(("campaign", "finalized"), new_status.clone());
        Ok(new_status)
    }

    /// Returns the current campaign status.
    pub fn status(env: Env) -> Status {
        env.storage().instance().get(&DataKey::Status).unwrap()
    }

    /// Withdraw raised funds — only callable by the creator after `Succeeded`.
    pub fn withdraw(env: Env) -> Result<(), ContractError> {
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Succeeded {
            panic!("campaign must be in Succeeded state to withdraw");
        }

        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        creator.require_auth();

        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();
        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_address);

        let platform_config: Option<PlatformConfig> =
            env.storage().instance().get(&DataKey::PlatformConfig);

        let creator_payout = if let Some(config) = platform_config {
            let fee = total
                .checked_mul(config.fee_bps as i128)
                .expect("fee calculation overflow")
                .checked_div(10_000)
                .expect("fee division by zero");
            if fee > 0 {
                token_client.transfer(&env.current_contract_address(), &config.address, &fee);
                withdraw_event_emission::emit_fee_transferred(&env, &config.address, fee);
            }
            total.checked_sub(fee).expect("creator payout underflow")
        } else {
            total
        };

        token_client.transfer(&env.current_contract_address(), &creator, &creator_payout);
        env.storage().instance().set(&DataKey::TotalRaised, &0i128);

        let nft_contract: Option<Address> = env.storage().instance().get(&DataKey::NFTContract);
        let nft_minted_count = mint_nfts_in_batch(&env, &nft_contract);
        emit_withdrawal_event(&env, &creator, creator_payout, nft_minted_count);
        Ok(())
    }

    /// Claim a refund for a single contributor (pull-based).
    pub fn refund_single(env: Env, contributor: Address) -> Result<(), ContractError> {
        contributor.require_auth();
        let amount = validate_refund_preconditions(&env, &contributor)?;
        execute_refund_single(&env, &contributor, amount)
    }

    /// Check if a refund is available for the given contributor (view function).
    pub fn refund_available(env: Env, contributor: Address) -> Result<i128, ContractError> {
        validate_refund_preconditions(&env, &contributor)
    }

    /// Cancel the campaign — callable only by the creator while Active.
    pub fn cancel(env: Env) {
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            panic!("campaign is not active");
        }

        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        creator.require_auth();

        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_address);

        let contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or_else(|| Vec::new(&env));

        for contributor in contributors.iter() {
            let contribution_key = DataKey::Contribution(contributor.clone());
            let amount: i128 = env
                .storage()
                .persistent()
                .get(&contribution_key)
                .unwrap_or(0);
            if amount > 0 {
                env.storage().persistent().set(&contribution_key, &0i128);
                refund_single_transfer(
                    &token_client,
                    &env.current_contract_address(),
                    &contributor,
                    amount,
                );
            }
        }

        env.storage().instance().set(&DataKey::TotalRaised, &0i128);
        env.storage().instance().set(&DataKey::Status, &Status::Cancelled);
    }

    /// Upgrade the contract WASM — admin-only.
    pub fn upgrade(env: Env, new_wasm_hash: soroban_sdk::BytesN<32>) {
        let admin = admin_upgrade_mechanism::validate_admin_upgrade(&env);
        admin_upgrade_mechanism::perform_upgrade(&env, new_wasm_hash.clone());
        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "upgrade"), admin),
            new_wasm_hash,
        );
    }

    /// Update campaign metadata — only callable by the creator while Active.
    pub fn update_metadata(
        env: Env,
        creator: Address,
        title: Option<String>,
        description: Option<String>,
        socials: Option<String>,
    ) {
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            panic!("campaign is not active");
        }

        let stored_creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        if creator != stored_creator {
            panic!("not authorized");
        }
        creator.require_auth();

        let mut updated_fields: Vec<Symbol> = Vec::new(&env);

        if let Some(new_title) = title {
            env.storage().instance().set(&DataKey::Title, &new_title);
            updated_fields.push_back(Symbol::new(&env, "title"));
        }
        if let Some(new_description) = description {
            env.storage().instance().set(&DataKey::Description, &new_description);
            updated_fields.push_back(Symbol::new(&env, "description"));
        }
        if let Some(new_socials) = socials {
            env.storage().instance().set(&DataKey::SocialLinks, &new_socials);
            updated_fields.push_back(Symbol::new(&env, "socials"));
        }

        env.events().publish(
            (Symbol::new(&env, "metadata_updated"), creator.clone()),
            updated_fields,
        );
    }

    /// Add a roadmap item — only callable by the creator.
    pub fn add_roadmap_item(env: Env, date: u64, description: String) {
        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        creator.require_auth();

        if date <= env.ledger().timestamp() {
            panic!("date must be in the future");
        }
        if description.is_empty() {
            panic!("description cannot be empty");
        }

        let mut roadmap: Vec<RoadmapItem> = env
            .storage()
            .instance()
            .get(&DataKey::Roadmap)
            .unwrap_or_else(|| Vec::new(&env));

        roadmap.push_back(RoadmapItem { date, description: description.clone() });
        env.storage().instance().set(&DataKey::Roadmap, &roadmap);
        env.events().publish(("campaign", "roadmap_item_added"), (date, description));
    }

    /// Returns all roadmap items.
    pub fn roadmap(env: Env) -> Vec<RoadmapItem> {
        env.storage()
            .instance()
            .get(&DataKey::Roadmap)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Add a stretch goal milestone — only callable by the creator.
    pub fn add_stretch_goal(env: Env, milestone: i128) {
        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        creator.require_auth();

        let goal: i128 = env.storage().instance().get(&DataKey::Goal).unwrap();
        if milestone <= goal {
            panic!("stretch goal must be greater than primary goal");
        }

        let mut stretch_goals: Vec<i128> = env
            .storage()
            .instance()
            .get(&DataKey::StretchGoals)
            .unwrap_or_else(|| Vec::new(&env));
        stretch_goals.push_back(milestone);
        env.storage().instance().set(&DataKey::StretchGoals, &stretch_goals);
    }

    /// Returns the next unmet stretch goal milestone (0 if none).
    pub fn current_milestone(env: Env) -> i128 {
        let total_raised: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalRaised)
            .unwrap_or(0);
        let stretch_goals: Vec<i128> = env
            .storage()
            .instance()
            .get(&DataKey::StretchGoals)
            .unwrap_or_else(|| Vec::new(&env));
        for milestone in stretch_goals.iter() {
            if total_raised < milestone {
                return milestone;
            }
        }
        0
    }

    // ── View functions ────────────────────────────────────────────────────────

    pub fn total_raised(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalRaised).unwrap_or(0)
    }

    pub fn goal(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::Goal).unwrap()
    }

    pub fn bonus_goal(env: Env) -> Option<i128> {
        env.storage().instance().get(&DataKey::BonusGoal)
    }

    pub fn bonus_goal_description(env: Env) -> Option<String> {
        env.storage().instance().get(&DataKey::BonusGoalDescription)
    }

    pub fn bonus_goal_reached(env: Env) -> bool {
        let total_raised: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalRaised)
            .unwrap_or(0);
        if let Some(bg) = env.storage().instance().get::<_, i128>(&DataKey::BonusGoal) {
            total_raised >= bg
        } else {
            false
        }
    }

    pub fn bonus_goal_progress_bps(env: Env) -> u32 {
        let total_raised: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalRaised)
            .unwrap_or(0);
        if let Some(bg) = env.storage().instance().get::<_, i128>(&DataKey::BonusGoal) {
            campaign_goal_minimum::compute_progress_bps(total_raised, bg)
        } else {
            0
        }
    }

    pub fn deadline(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::Deadline).unwrap()
    }

    pub fn contribution(env: Env, contributor: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Contribution(contributor))
            .unwrap_or(0)
    }

    pub fn min_contribution(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::MinContribution).unwrap()
    }

    pub fn get_stats(env: Env) -> CampaignStats {
        let total_raised: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalRaised)
            .unwrap_or(0);
        let goal: i128 = env.storage().instance().get(&DataKey::Goal).unwrap();
        let contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or_else(|| Vec::new(&env));

        let progress_bps = campaign_goal_minimum::compute_progress_bps(total_raised, goal);
        let contributor_count = contributors.len();
        let (average_contribution, largest_contribution) = if contributor_count == 0 {
            (0, 0)
        } else {
            let average = total_raised / contributor_count as i128;
            let mut largest = 0i128;
            for contributor in contributors.iter() {
                let amount: i128 = env
                    .storage()
                    .persistent()
                    .get(&DataKey::Contribution(contributor))
                    .unwrap_or(0);
                if amount > largest {
                    largest = amount;
                }
            }
            (average, largest)
        };

        CampaignStats {
            total_raised,
            goal,
            progress_bps,
            contributor_count,
            average_contribution,
            largest_contribution,
        }
    }

    pub fn title(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Title)
            .unwrap_or_else(|| String::from_str(&env, ""))
    }

    pub fn description(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Description)
            .unwrap_or_else(|| String::from_str(&env, ""))
    }

    pub fn socials(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::SocialLinks)
            .unwrap_or_else(|| String::from_str(&env, ""))
    }

    pub fn version(_env: Env) -> u32 {
        CONTRACT_VERSION
    }

    pub fn token(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Token).unwrap()
    }

    pub fn token_decimals(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::TokenDecimals).unwrap_or(7)
    }

    pub fn nft_contract(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::NFTContract)
    }
}
