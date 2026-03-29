//! # crowdfund_initialize_function
//!
//! @title   CrowdfundInitializeFunction — Validated initialization logic for
//!          the crowdfund contract.
//!
//! @notice  Extracts and standardizes `initialize()` logic from `lib.rs` into
//!          a single, auditable location.  Provides:
//!          - A validated `InitParams` struct.
//!          - Pure validation helpers returning typed `ContractError` values.
//!          - A deterministic, single-pass initialization flow with clear
//!            checks → effects → storage write ordering.
//!
//! ## Security Assumptions
//!
//! 1. Re-initialization guard uses `DataKey::Creator` as the sentinel.
//! 2. `creator.require_auth()` is called before any storage write.
//! 3. `goal >= MIN_GOAL_AMOUNT` prevents zero-goal campaigns.
//! 4. `min_contribution >= MIN_CONTRIBUTION_AMOUNT` prevents dust attacks.
//! 5. `deadline >= now + MIN_DEADLINE_OFFSET` ensures the campaign is live.
//! 6. `fee_bps <= MAX_PLATFORM_FEE_BPS` caps the platform take.
//! 7. `bonus_goal > goal` prevents a bonus goal met at launch.
//! 8. All validations complete before the first storage write.

#![allow(dead_code)]

use soroban_sdk::{Address, Env, String, Vec};

use crate::campaign_goal_minimum::{
    validate_deadline, validate_goal, validate_min_contribution, validate_platform_fee,
};
use crate::{ContractError, DataKey, PlatformConfig, RoadmapItem, Status};

// ── InitParams ────────────────────────────────────────────────────────────────

/// All parameters required to initialize a crowdfund campaign.
#[derive(Clone)]
pub struct InitParams {
    pub admin: Address,
    pub creator: Address,
    pub token: Address,
    pub goal: i128,
    pub deadline: u64,
    pub min_contribution: i128,
    pub platform_config: Option<PlatformConfig>,
    pub bonus_goal: Option<i128>,
    pub bonus_goal_description: Option<String>,
}

// ── Validation helpers ────────────────────────────────────────────────────────

/// Validates that `bonus_goal`, when present, is strictly greater than `goal`.
#[inline]
pub fn validate_bonus_goal(bonus_goal: Option<i128>, goal: i128) -> Result<(), ContractError> {
    if let Some(bg) = bonus_goal {
        if bg <= goal {
            return Err(ContractError::InvalidBonusGoal);
        }
    }
    Ok(())
}

/// Validates all `InitParams` fields in a single pass.
pub fn validate_init_params(env: &Env, params: &InitParams) -> Result<(), ContractError> {
    validate_goal(params.goal).map_err(|_| ContractError::InvalidGoal)?;
    validate_min_contribution(params.min_contribution)
        .map_err(|_| ContractError::InvalidMinContribution)?;
    validate_deadline(env.ledger().timestamp(), params.deadline)
        .map_err(|_| ContractError::DeadlineTooSoon)?;
    if let Some(ref config) = params.platform_config {
        validate_platform_fee(config.fee_bps).map_err(|_| ContractError::InvalidPlatformFee)?;
    }
    validate_bonus_goal(params.bonus_goal, params.goal)?;
    Ok(())
}

// ── Core initialization logic ─────────────────────────────────────────────────

/// Executes the full campaign initialization flow.
///
/// Ordering guarantee:
/// 1. Re-initialization guard (read-only check).
/// 2. Creator authentication (`require_auth`).
/// 3. Full parameter validation (no storage writes yet).
/// 4. Storage writes (all-or-nothing within the transaction).
/// 5. Event emission.
pub fn execute_initialize(env: &Env, params: InitParams) -> Result<(), ContractError> {
    // 1. Re-initialization guard
    if env.storage().instance().has(&DataKey::Creator) {
        return Err(ContractError::AlreadyInitialized);
    }

    // 2. Creator authentication
    params.creator.require_auth();

    // 3. Parameter validation
    validate_init_params(env, &params)?;

    // 4. Storage writes
    env.storage().instance().set(&DataKey::Admin, &params.admin);
    env.storage().instance().set(&DataKey::Creator, &params.creator);
    env.storage().instance().set(&DataKey::Token, &params.token);
    env.storage().instance().set(&DataKey::Goal, &params.goal);
    env.storage().instance().set(&DataKey::Deadline, &params.deadline);
    env.storage().instance().set(&DataKey::MinContribution, &params.min_contribution);
    env.storage().instance().set(&DataKey::TotalRaised, &0i128);
    env.storage().instance().set(&DataKey::BonusGoalReachedEmitted, &false);
    env.storage().instance().set(&DataKey::Status, &Status::Active);

    if let Some(ref config) = params.platform_config {
        env.storage().instance().set(&DataKey::PlatformConfig, config);
    }
    if let Some(bg) = params.bonus_goal {
        env.storage().instance().set(&DataKey::BonusGoal, &bg);
    }
    if let Some(ref bg_desc) = params.bonus_goal_description {
        env.storage().instance().set(&DataKey::BonusGoalDescription, bg_desc);
    }

    let empty_contributors: Vec<Address> = Vec::new(env);
    env.storage().persistent().set(&DataKey::Contributors, &empty_contributors);

    let empty_roadmap: Vec<RoadmapItem> = Vec::new(env);
    env.storage().instance().set(&DataKey::Roadmap, &empty_roadmap);

    // 5. Event emission
    env.events().publish(
        (
            soroban_sdk::Symbol::new(env, "campaign"),
            soroban_sdk::Symbol::new(env, "initialized"),
        ),
        (
            params.creator.clone(),
            params.token.clone(),
            params.goal,
            params.deadline,
            params.min_contribution,
        ),
    );

    Ok(())
}

// ── Error description helpers ─────────────────────────────────────────────────

/// Returns a human-readable description for an `initialize()`-related error code.
pub fn describe_init_error(code: u32) -> &'static str {
    match code {
        1 => "Contract is already initialized",
        8 => "Campaign goal must be at least 1",
        9 => "Minimum contribution must be at least 1",
        10 => "Deadline must be at least 60 seconds in the future",
        11 => "Platform fee cannot exceed 100% (10,000 bps)",
        12 => "Bonus goal must be strictly greater than the primary goal",
        _ => "Unknown initialization error",
    }
}

/// Returns `true` if the error code corresponds to a correctable input error.
pub fn is_init_error_retryable(code: u32) -> bool {
    matches!(code, 8 | 9 | 10 | 11 | 12)
}

/// Re-exports `MIN_GOAL_AMOUNT` for callers that only import this module.
pub use crate::campaign_goal_minimum::MIN_GOAL_AMOUNT as INIT_MIN_GOAL_AMOUNT;
