//! # campaign_goal_minimum
//!
//! @title   CampaignGoalMinimum — Validation helpers for campaign goal and
//!          contribution thresholds.
//!
//! @notice  This module enforces minimum thresholds for campaign goals,
//!          minimum contributions, deadlines, and platform fees.  All
//!          validation is pure (no storage reads) so it can be used in
//!          off-chain tooling and property-based tests without a full
//!          Soroban environment.
//!
//! ## Security Rationale
//!
//! A campaign goal below `MIN_GOAL_AMOUNT` (1 token unit) would allow a
//! zero-goal campaign to be immediately "successful" after any contribution,
//! letting the creator drain funds with no real commitment.  A zero
//! `min_contribution` allows dust attacks that pollute the contributor list
//! and waste ledger storage.
//!
//! ## Integer-Overflow Safety
//!
//! All comparisons in this module are single signed/unsigned integer
//! comparisons — no arithmetic is performed, so overflow is impossible.
//! `validate_deadline` uses `saturating_add` to guard against `u64` overflow
//! when `now` is near `u64::MAX`.

// ── Constants ─────────────────────────────────────────────────────────────────

/// Minimum allowed campaign goal (in token units).
///
/// @notice A goal of 0 would make the campaign immediately successful after
///         any contribution, allowing the creator to drain funds trivially.
pub const MIN_GOAL_AMOUNT: i128 = 1;

/// Minimum allowed `min_contribution` value (in token units).
///
/// @notice A zero minimum contribution allows dust attacks that waste ledger
///         storage and inflate the contributor count.
pub const MIN_CONTRIBUTION_AMOUNT: i128 = 1;

/// Minimum number of seconds the deadline must be in the future at
/// initialization time.
///
/// @notice Ensures the campaign is live for at least one ledger close
///         interval (~5 s on Stellar mainnet), giving contributors time to act.
pub const MIN_DEADLINE_OFFSET: u64 = 60;

/// Maximum platform fee in basis points (10 000 bps = 100 %).
///
/// @notice A fee above 100 % would leave the creator with a negative payout,
///         which is economically nonsensical and would cause an underflow.
pub const MAX_PLATFORM_FEE_BPS: u32 = 10_000;

/// Scale factor used when computing progress in basis points.
pub const PROGRESS_BPS_SCALE: i128 = 10_000;

/// Maximum value returned by `compute_progress_bps` (100 % = 10 000 bps).
pub const MAX_PROGRESS_BPS: u32 = 10_000;

// ── Validation helpers ────────────────────────────────────────────────────────

/// Validates that `goal` meets the minimum threshold.
///
/// @param  goal  The proposed campaign goal in token units.
/// @return       `Ok(())` if `goal >= MIN_GOAL_AMOUNT`, `Err` otherwise.
///
/// @dev    Returns `&'static str` so this helper stays free of the contract's
///         error type and can be used in off-chain tooling.
#[inline]
pub fn validate_goal(goal: i128) -> Result<(), &'static str> {
    if goal < MIN_GOAL_AMOUNT {
        return Err("goal must be at least MIN_GOAL_AMOUNT");
    }
    Ok(())
}

/// Validates that `goal_amount` meets the minimum threshold, returning a typed
/// [`crate::ContractError::InvalidGoal`] on failure.
///
/// @notice  This is the on-chain enforcement entry point.  Call this inside
///          `initialize()` before persisting any campaign state so that a
///          below-threshold goal is rejected atomically with no side-effects.
///
/// @dev     `_env` is accepted for API consistency and to allow future
///          ledger-aware threshold logic without a breaking signature change.
///
/// @param  _env         The Soroban environment (reserved for future use).
/// @param  goal_amount  The proposed campaign goal in token units.
/// @return              `Ok(())` if `goal_amount >= MIN_GOAL_AMOUNT`,
///                      `Err(ContractError::InvalidGoal)` otherwise.
#[inline]
pub fn validate_goal_amount(
    _env: &soroban_sdk::Env,
    goal_amount: i128,
) -> Result<(), crate::ContractError> {
    if goal_amount < MIN_GOAL_AMOUNT {
        return Err(crate::ContractError::InvalidGoal);
    }
    Ok(())
}

/// Validates that `min_contribution` meets the minimum floor.
///
/// @param  min_contribution  The proposed minimum contribution in token units.
/// @return                   `Ok(())` if valid, `Err` otherwise.
#[inline]
pub fn validate_min_contribution(min_contribution: i128) -> Result<(), &'static str> {
    if min_contribution < MIN_CONTRIBUTION_AMOUNT {
        return Err("min_contribution must be at least MIN_CONTRIBUTION_AMOUNT");
    }
    Ok(())
}

/// Validates that `deadline` is sufficiently far in the future.
///
/// @param  now       Current ledger timestamp (seconds since epoch).
/// @param  deadline  Proposed campaign deadline (seconds since epoch).
/// @return           `Ok(())` if `deadline >= now + MIN_DEADLINE_OFFSET`, `Err` otherwise.
///
/// @dev    Uses `saturating_add` to prevent `u64` overflow when `now` is near
///         `u64::MAX`.
#[inline]
pub fn validate_deadline(now: u64, deadline: u64) -> Result<(), &'static str> {
    let min_deadline = now.saturating_add(MIN_DEADLINE_OFFSET);
    if deadline < min_deadline {
        return Err("deadline must be at least MIN_DEADLINE_OFFSET seconds in the future");
    }
    Ok(())
}

/// Validates that `fee_bps` does not exceed `MAX_PLATFORM_FEE_BPS`.
///
/// @param  fee_bps  Platform fee in basis points.
/// @return          `Ok(())` if `fee_bps <= MAX_PLATFORM_FEE_BPS`, `Err` otherwise.
#[inline]
pub fn validate_platform_fee(fee_bps: u32) -> Result<(), &'static str> {
    if fee_bps > MAX_PLATFORM_FEE_BPS {
        return Err("fee_bps must not exceed MAX_PLATFORM_FEE_BPS");
    }
    Ok(())
}

/// Computes campaign progress in basis points, capped at `MAX_PROGRESS_BPS`.
///
/// @param  total_raised  Total tokens raised so far.
/// @param  goal          Campaign funding goal.
/// @return               Progress in bps (0–10 000).  Returns 0 if `goal <= 0`.
///
/// @dev    Integer division truncates toward zero, so 1 token raised against a
///         1 000 000-token goal returns 0 bps (< 1 bps).
#[inline]
pub fn compute_progress_bps(total_raised: i128, goal: i128) -> u32 {
    if goal <= 0 {
        return 0;
    }
    let raw = (total_raised * PROGRESS_BPS_SCALE) / goal;
    if raw >= PROGRESS_BPS_SCALE {
        MAX_PROGRESS_BPS
    } else if raw < 0 {
        0
    } else {
        raw as u32
    }
}
