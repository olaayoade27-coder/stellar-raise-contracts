//! Contract State Size Limits
//!
//! Defines maximum size limits for all campaign-related on-chain state and
//! provides guard functions that return typed errors when limits are exceeded.
//!
//! ## Why limits matter
//! - **Resource efficiency**: caps ledger entry sizes, keeping state-rent predictable.
//! - **Frontend reliability**: the UI can query these constants to pre-validate inputs.
//! - **Scalability**: bounded collections prevent runaway storage growth.
//!
//! All byte constants are measured in UTF-8 bytes; count constants are item counts.

use soroban_sdk::{contract, contracterror, contractimpl, Address, Env, String, Vec};

use crate::{DataKey, RoadmapItem};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum length of any single string field (title, description, social links) in bytes.
/// @dev Shared limit used by `check_string_len`.
pub const MAX_STRING_LEN: u32 = 256;

/// Maximum number of unique contributors (and pledgers) tracked per campaign.
/// @dev Bounds both the `Contributors` and `Pledgers` persistent lists.
pub const MAX_CONTRIBUTORS: u32 = 128;

/// Maximum number of roadmap milestones stored per campaign.
pub const MAX_ROADMAP_ITEMS: u32 = 32;

/// Maximum number of stretch goals (milestones) stored per campaign.
pub const MAX_STRETCH_GOALS: u32 = 32;

// Legacy aliases kept for backward compatibility with contract_state_size.test.rs
pub const MAX_TITLE_LENGTH: u32 = MAX_STRING_LEN;
pub const MAX_DESCRIPTION_LENGTH: u32 = MAX_STRING_LEN;

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors returned when a state-size limit is exceeded.
///
/// Discriminants are stable and must not be renumbered — they are part of the
/// on-chain ABI.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StateSizeError {
    /// The `Contributors` or `Pledgers` list has reached `MAX_CONTRIBUTORS`.
    ContributorLimitExceeded = 100,
    /// The `Roadmap` list has reached `MAX_ROADMAP_ITEMS`.
    RoadmapLimitExceeded = 101,
    /// The `StretchGoals` list has reached `MAX_STRETCH_GOALS`.
    StretchGoalLimitExceeded = 102,
    /// A string field exceeds `MAX_STRING_LEN` bytes.
    StringTooLong = 103,
}

// ── Guard functions ───────────────────────────────────────────────────────────

/// Returns `Err(StringTooLong)` if `s` exceeds `MAX_STRING_LEN` bytes.
///
/// @param s  The string to validate.
/// @return   `Ok(())` when within bounds.
pub fn check_string_len(s: &String) -> Result<(), StateSizeError> {
    if s.len() > MAX_STRING_LEN {
        Err(StateSizeError::StringTooLong)
    } else {
        Ok(())
    }
}

/// Returns `Err(ContributorLimitExceeded)` if the `Contributors` list is full.
///
/// Reads `DataKey::Contributors` from persistent storage; treats a missing key
/// as an empty list (safe default).
///
/// @param env  The contract environment.
/// @return     `Ok(())` when below `MAX_CONTRIBUTORS`.
pub fn check_contributor_limit(env: &Env) -> Result<(), StateSizeError> {
    let count = env
        .storage()
        .persistent()
        .get::<DataKey, Vec<Address>>(&DataKey::Contributors)
        .map(|v| v.len())
        .unwrap_or(0);
    if count >= MAX_CONTRIBUTORS {
        Err(StateSizeError::ContributorLimitExceeded)
    } else {
        Ok(())
    }
}

/// Returns `Err(ContributorLimitExceeded)` if the `Pledgers` list is full.
///
/// Mirrors `check_contributor_limit` but reads `DataKey::Pledgers`.
///
/// @param env  The contract environment.
/// @return     `Ok(())` when below `MAX_CONTRIBUTORS`.
pub fn check_pledger_limit(env: &Env) -> Result<(), StateSizeError> {
    let count = env
        .storage()
        .persistent()
        .get::<DataKey, Vec<Address>>(&DataKey::Pledgers)
        .map(|v| v.len())
        .unwrap_or(0);
    if count >= MAX_CONTRIBUTORS {
        Err(StateSizeError::ContributorLimitExceeded)
    } else {
        Ok(())
    }
}

/// Returns `Err(RoadmapLimitExceeded)` if the `Roadmap` list is full.
///
/// Reads `DataKey::Roadmap` from instance storage.
///
/// @param env  The contract environment.
/// @return     `Ok(())` when below `MAX_ROADMAP_ITEMS`.
pub fn check_roadmap_limit(env: &Env) -> Result<(), StateSizeError> {
    let count = env
        .storage()
        .instance()
        .get::<DataKey, Vec<RoadmapItem>>(&DataKey::Roadmap)
        .map(|v| v.len())
        .unwrap_or(0);
    if count >= MAX_ROADMAP_ITEMS {
        Err(StateSizeError::RoadmapLimitExceeded)
    } else {
        Ok(())
    }
}

/// Returns `Err(StretchGoalLimitExceeded)` if the `StretchGoals` list is full.
///
/// Reads `DataKey::StretchGoals` from instance storage.
///
/// @param env  The contract environment.
/// @return     `Ok(())` when below `MAX_STRETCH_GOALS`.
pub fn check_stretch_goal_limit(env: &Env) -> Result<(), StateSizeError> {
    let count = env
        .storage()
        .instance()
        .get::<DataKey, Vec<i128>>(&DataKey::StretchGoals)
        .map(|v| v.len())
        .unwrap_or(0);
    if count >= MAX_STRETCH_GOALS {
        Err(StateSizeError::StretchGoalLimitExceeded)
    } else {
        Ok(())
    }
}

// ── Capacity / length validators (called from lib.rs and other modules) ───────

/// Returns `Err(ContributorLimitExceeded)` if `count >= MAX_CONTRIBUTORS`.
/// @param count  Current length of the contributors list.
pub fn validate_contributor_capacity(count: u32) -> Result<(), StateSizeError> {
    if count >= MAX_CONTRIBUTORS {
        Err(StateSizeError::ContributorLimitExceeded)
    } else {
        Ok(())
    }
}

/// Returns `Err(ContributorLimitExceeded)` if `count >= MAX_CONTRIBUTORS`.
/// @param count  Current length of the pledgers list.
pub fn validate_pledger_capacity(count: u32) -> Result<(), StateSizeError> {
    if count >= MAX_CONTRIBUTORS {
        Err(StateSizeError::ContributorLimitExceeded)
    } else {
        Ok(())
    }
}

/// Returns `Err(RoadmapLimitExceeded)` if `count >= MAX_ROADMAP_ITEMS`.
/// @param count  Current length of the roadmap list.
pub fn validate_roadmap_capacity(count: u32) -> Result<(), StateSizeError> {
    if count >= MAX_ROADMAP_ITEMS {
        Err(StateSizeError::RoadmapLimitExceeded)
    } else {
        Ok(())
    }
}

/// Returns `Err(StretchGoalLimitExceeded)` if `count >= MAX_STRETCH_GOALS`.
/// @param count  Current length of the stretch goals list.
pub fn validate_stretch_goal_capacity(count: u32) -> Result<(), StateSizeError> {
    if count >= MAX_STRETCH_GOALS {
        Err(StateSizeError::StretchGoalLimitExceeded)
    } else {
        Ok(())
    }
}

/// Returns `Err(StringTooLong)` if `title` exceeds `MAX_STRING_LEN`.
pub fn validate_title(title: &String) -> Result<(), StateSizeError> {
    check_string_len(title)
}

/// Returns `Err(StringTooLong)` if `description` exceeds `MAX_STRING_LEN`.
pub fn validate_description(description: &String) -> Result<(), StateSizeError> {
    check_string_len(description)
}

/// Returns `Err(StringTooLong)` if `social_links` exceeds `MAX_STRING_LEN`.
pub fn validate_social_links(social_links: &String) -> Result<(), StateSizeError> {
    check_string_len(social_links)
}

/// Returns `Err(StringTooLong)` if `description` (roadmap item) exceeds `MAX_STRING_LEN`.
pub fn validate_roadmap_description(description: &String) -> Result<(), StateSizeError> {
    check_string_len(description)
}

/// Returns `Err(StringTooLong)` if `desc` (bonus goal description) exceeds `MAX_STRING_LEN`.
pub fn validate_bonus_goal_description(desc: &String) -> Result<(), StateSizeError> {
    check_string_len(desc)
}

/// Returns `Err(StringTooLong)` if the combined metadata length exceeds the aggregate limit.
/// @param title_len        Length of the title string in bytes.
/// @param description_len  Length of the description string in bytes.
/// @param socials_len      Length of the social links string in bytes.
pub fn validate_metadata_total_length(
    title_len: u32,
    description_len: u32,
    socials_len: u32,
) -> Result<(), StateSizeError> {
    const AGGREGATE_LIMIT: u32 = MAX_TITLE_LENGTH + MAX_DESCRIPTION_LENGTH + MAX_STRING_LEN;
    if title_len
        .saturating_add(description_len)
        .saturating_add(socials_len)
        > AGGREGATE_LIMIT
    {
        Err(StateSizeError::StringTooLong)
    } else {
        Ok(())
    }
}

// ── Queryable contract (used by contract_state_size.test.rs) ─────────────────

/// Standalone contract that exposes state-size constants over the Soroban ABI.
/// @dev Primarily used by the frontend to query limits without off-chain config.
#[contract]
pub struct ContractStateSize;

#[contractimpl]
impl ContractStateSize {
    /// Returns `MAX_TITLE_LENGTH`.
    pub fn max_title_length(_env: Env) -> u32 {
        MAX_TITLE_LENGTH
    }
    /// Returns `MAX_DESCRIPTION_LENGTH`.
    pub fn max_description_length(_env: Env) -> u32 {
        MAX_DESCRIPTION_LENGTH
    }
    /// Returns `MAX_STRING_LEN` (social links limit).
    pub fn max_social_links_length(_env: Env) -> u32 {
        MAX_STRING_LEN
    }
    /// Returns `MAX_CONTRIBUTORS`.
    pub fn max_contributors(_env: Env) -> u32 {
        MAX_CONTRIBUTORS
    }
    /// Returns `MAX_ROADMAP_ITEMS`.
    pub fn max_roadmap_items(_env: Env) -> u32 {
        MAX_ROADMAP_ITEMS
    }
    /// Returns `MAX_STRETCH_GOALS`.
    pub fn max_stretch_goals(_env: Env) -> u32 {
        MAX_STRETCH_GOALS
    }
}

    /// Returns `true` if `title` length is within `MAX_TITLE_LENGTH`.
    pub fn validate_title(_env: Env, title: String) -> bool {
        title.len() <= MAX_TITLE_LENGTH
    }
    /// Returns `true` if `description` length is within `MAX_DESCRIPTION_LENGTH`.
    pub fn validate_description(_env: Env, description: String) -> bool {
        description.len() <= MAX_DESCRIPTION_LENGTH
    }
    /// Returns `true` if `total_len` is within the aggregate metadata limit.
    pub fn validate_metadata_aggregate(_env: Env, total_len: u32) -> bool {
        total_len <= MAX_TITLE_LENGTH + MAX_DESCRIPTION_LENGTH + MAX_STRING_LEN
    }
}

/// Legacy constant for backwards compatibility (MAX_STRING_LEN = MAX_DESCRIPTION_LENGTH)
pub const MAX_STRING_LEN: u32 = MAX_DESCRIPTION_LENGTH;

/// Legacy function for backwards compatibility.
/// Checks that a string does not exceed MAX_DESCRIPTION_LENGTH bytes.
///
/// @param s The string to validate.
/// @return Ok(()) when within limits, Err otherwise.
pub fn check_string_len(s: &String) -> Result<(), StateSizeError> {
    if s.len() > MAX_STRING_LEN {
        return Err(StateSizeError::StringTooLong);
//! # contract_state_size
//!
//! @title   ContractStateSize — Centralized limits for variable-size contract state
//! @notice  Bounds metadata strings and collection growth so the crowdfund
//!          contract cannot accumulate unreviewed, unbounded state over time.
//! @dev     The helpers in this module are pure and return `Result<(), &'static str>`
//!          so they can be reused from contract logic, tests, and off-chain
//!          tooling without coupling to `ContractError`.
//!
//! ## Why these limits exist
//!
//! The crowdfund contract stores several fields whose size is controlled by
//! user input:
//!
//! 1. Metadata strings (`title`, `description`, `socials`, bonus-goal text)
//! 2. Contributor and pledger address lists
//! 3. Roadmap item descriptions
//! 4. Stretch-goal vectors
//!
//! Without explicit bounds, these values can grow until:
//!
//! - CI tests become slower and more memory-intensive
//! - Storage growth becomes harder to reason about during review
//! - Contract calls that iterate over vectors become less predictable
//! - Off-chain tooling sees inconsistent or excessively large payloads
//!
//! ## Security assumptions
//!
//! 1. Fixed maxima make worst-case state growth auditable.
//! 2. Rejecting oversize writes before persistence prevents silent storage bloat.
//! 3. Bounding collection counts reduces gas and event-surface risk in flows
//!    that later read or iterate over those collections.
//! 4. Metadata limits are sized for practical UX while preventing abuse via
//!    arbitrarily large strings.

use soroban_sdk::String;

/// Maximum number of contributor addresses stored in the contributor index.
///
/// @dev Contributions themselves remain stored per-address. This limit bounds
///      the enumerated contributor list used by view methods and reward flows.
pub const MAX_CONTRIBUTORS: u32 = 128;

/// Maximum number of pledger addresses stored in the pledger index.
pub const MAX_PLEDGERS: u32 = 128;

/// Maximum number of roadmap items allowed for a campaign.
pub const MAX_ROADMAP_ITEMS: u32 = 32;

/// Maximum number of stretch-goal milestones allowed for a campaign.
pub const MAX_STRETCH_GOALS: u32 = 32;

/// Maximum UTF-8 byte length for the campaign title.
pub const MAX_TITLE_LENGTH: u32 = 128;

/// Maximum UTF-8 byte length for the campaign description.
pub const MAX_DESCRIPTION_LENGTH: u32 = 2_048;

/// Maximum UTF-8 byte length for the social-links field.
pub const MAX_SOCIAL_LINKS_LENGTH: u32 = 512;

/// Maximum UTF-8 byte length for the optional bonus-goal description.
pub const MAX_BONUS_GOAL_DESCRIPTION_LENGTH: u32 = 280;

/// Maximum UTF-8 byte length for a roadmap item description.
pub const MAX_ROADMAP_DESCRIPTION_LENGTH: u32 = 280;

/// Maximum combined metadata footprint for title + description + socials.
///
/// @dev This budget is intentionally stricter than the sum of the individual
///      field maxima so callers cannot max out every metadata field at once.
pub const MAX_METADATA_TOTAL_LENGTH: u32 = 2_304;

#[inline]
fn validate_string_length(
    value: &String,
    max_length: u32,
    error: &'static str,
) -> Result<(), &'static str> {
    if value.len() > max_length {
        return Err(error);
//! # Contract State Size Limits
//!
//! This module enforces upper-bound limits on the size of unbounded collections
//! stored in contract state to prevent:
//!
//! - **DoS via state bloat**: an attacker flooding the contributors or roadmap
//!   lists until operations become too expensive to execute.
//! - **Gas exhaustion**: iteration over an unbounded `Vec` in `withdraw`,
//!   `refund`, or `collect_pledges` can exceed Soroban resource limits.
//! - **Ledger entry size violations**: Soroban enforces a hard cap on the
//!   serialised size of each ledger entry; exceeding it causes a host panic.
//!
//! ## Security Assumptions
//!
//! 1. `MAX_CONTRIBUTORS` caps the `Contributors` and `Pledgers` persistent
//!    lists.  Any `contribute` or `pledge` call that would push the list past
//!    this limit is rejected with [`ContractError::StateSizeLimitExceeded`].
//! 2. `MAX_ROADMAP_ITEMS` caps the `Roadmap` instance list.
//! 3. `MAX_STRING_LEN` caps every user-supplied `String` field (title,
//!    description, social links, roadmap description) to prevent oversized
//!    ledger entries.
//! 4. `MAX_STRETCH_GOALS` caps the `StretchGoals` list.
//!
//! ## Limits (rationale)
//!
//! | Constant              | Value | Rationale                                      |
//! |-----------------------|-------|------------------------------------------------|
//! | `MAX_CONTRIBUTORS`    | 1 000 | Keeps `withdraw` / `refund` batch within gas   |
//! | `MAX_ROADMAP_ITEMS`   |    20 | Cosmetic list; no operational iteration needed |
//! | `MAX_STRETCH_GOALS`   |    10 | Small advisory list                            |
//! | `MAX_STRING_LEN`      |   256 | Prevents oversized instance-storage entries    |

#![allow(missing_docs)]

use soroban_sdk::{contracterror, Env, String, Vec};

use crate::DataKey;

// ── Limits ───────────────────────────────────────────────────────────────────

/// Maximum number of unique contributors tracked on-chain.
pub const MAX_CONTRIBUTORS: u32 = 128;

/// Maximum number of unique pledgers tracked on-chain.
pub const MAX_PLEDGERS: u32 = 128;

/// Maximum number of roadmap items stored in instance storage.
pub const MAX_ROADMAP_ITEMS: u32 = 32;

/// Maximum number of stretch-goal milestones.
pub const MAX_STRETCH_GOALS: u32 = 32;

/// Maximum campaign title length in bytes.
pub const MAX_TITLE_LENGTH: u32 = 128;
/// Maximum campaign description length in bytes.
pub const MAX_DESCRIPTION_LENGTH: u32 = 2_048;
/// Maximum social-links payload length in bytes.
pub const MAX_SOCIAL_LINKS_LENGTH: u32 = 512;
/// Maximum bonus-goal description length in bytes.
pub const MAX_BONUS_GOAL_DESCRIPTION_LENGTH: u32 = 280;
/// Maximum roadmap item description length in bytes.
pub const MAX_ROADMAP_DESCRIPTION_LENGTH: u32 = 280;
/// Maximum combined metadata budget (`title + description + socials`) in bytes.
pub const MAX_METADATA_TOTAL_LENGTH: u32 = 2_304;
/// Backward-compatible generic string limit used by legacy tests/helpers.
pub const MAX_STRING_LEN: u32 = 256;

// ── Error ─────────────────────────────────────────────────────────────────────

/// Returned when a state-size limit would be exceeded.
///
/// @notice Callers should treat this as a permanent rejection for the current
///         campaign state; the limit will not change without a contract upgrade.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StateSizeError {
    /// The contributors / pledgers list is full.
    ContributorLimitExceeded = 100,
    /// The roadmap list is full.
    RoadmapLimitExceeded = 101,
    /// The stretch-goals list is full.
    StretchGoalLimitExceeded = 102,
    /// A string field exceeds `MAX_STRING_LEN` bytes.
    StringTooLong = 103,
}

impl core::fmt::Display for StateSizeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StateSizeError::ContributorLimitExceeded => {
                f.write_str("contributor limit exceeded")
            }
            StateSizeError::RoadmapLimitExceeded => f.write_str("roadmap limit exceeded"),
            StateSizeError::StretchGoalLimitExceeded => {
                f.write_str("stretch goal limit exceeded")
            }
            StateSizeError::StringTooLong => f.write_str("string too long"),
        }
    }
}

// ── Validation helpers ────────────────────────────────────────────────────────

/// Validate title length.
pub fn validate_title(value: &String) -> Result<(), &'static str> {
    if value.len() > MAX_TITLE_LENGTH {
        return Err("title exceeds MAX_TITLE_LENGTH bytes".into());
    }
    Ok(())
}

/// Validate description length.
pub fn validate_description(value: &String) -> Result<(), &'static str> {
    if value.len() > MAX_DESCRIPTION_LENGTH {
        return Err("description exceeds MAX_DESCRIPTION_LENGTH bytes".into());
    }
    Ok(())
}

/// Validate social links length.
pub fn validate_social_links(value: &String) -> Result<(), &'static str> {
    if value.len() > MAX_SOCIAL_LINKS_LENGTH {
        return Err("social links exceed MAX_SOCIAL_LINKS_LENGTH bytes".into());
    }
    Ok(())
}

/// Validate bonus goal description length.
pub fn validate_bonus_goal_description(value: &String) -> Result<(), &'static str> {
    if value.len() > MAX_BONUS_GOAL_DESCRIPTION_LENGTH {
        return Err(
            "bonus goal description exceeds MAX_BONUS_GOAL_DESCRIPTION_LENGTH bytes".into(),
        );
    }
    Ok(())
}

/// Validate roadmap item description length.
pub fn validate_roadmap_description(value: &String) -> Result<(), &'static str> {
    if value.len() > MAX_ROADMAP_DESCRIPTION_LENGTH {
        return Err("roadmap description exceeds MAX_ROADMAP_DESCRIPTION_LENGTH bytes".into());
    }
    Ok(())
}

/// Validate metadata aggregate length.
pub fn validate_metadata_total_length(
    title_len: u32,
    description_len: u32,
    socials_len: u32,
) -> Result<(), &'static str> {
    let sum = title_len
        .checked_add(description_len)
        .and_then(|v| v.checked_add(socials_len))
        .ok_or("metadata exceeds MAX_METADATA_TOTAL_LENGTH bytes")?;
    if sum > MAX_METADATA_TOTAL_LENGTH {
        return Err("metadata exceeds MAX_METADATA_TOTAL_LENGTH bytes".into());
    }
    Ok(())
}

/// Validate contributor index capacity before append.
pub fn validate_contributor_capacity(len: u32) -> Result<(), &'static str> {
    if len >= MAX_CONTRIBUTORS {
        return Err("contributors exceed MAX_CONTRIBUTORS".into());
    }
    Ok(())
}

/// Validate pledger index capacity before append.
pub fn validate_pledger_capacity(len: u32) -> Result<(), &'static str> {
    if len >= MAX_PLEDGERS {
        return Err("pledgers exceed MAX_PLEDGERS".into());
    }
    Ok(())
}

/// Validate roadmap capacity before append.
pub fn validate_roadmap_capacity(len: u32) -> Result<(), &'static str> {
    if len >= MAX_ROADMAP_ITEMS {
        return Err("roadmap exceeds MAX_ROADMAP_ITEMS".into());
    }
    Ok(())
}

/// Validate stretch-goal capacity before append.
pub fn validate_stretch_goal_capacity(len: u32) -> Result<(), &'static str> {
    if len >= MAX_STRETCH_GOALS {
        return Err("stretch goals exceed MAX_STRETCH_GOALS".into());
    }
    Ok(())
}

/// Assert that `s` does not exceed [`MAX_STRING_LEN`] bytes.
///
/// @param s The string to validate.
/// @return `Ok(())` when within limits, `Err(StateSizeError::StringTooLong)` otherwise.
pub fn check_string_len(s: &String) -> Result<(), StateSizeError> {
    if s.len() > MAX_STRING_LEN {
        return Err(StateSizeError::StringTooLong);
    }
    Ok(())
}

/// Checks stretch goal limit from storage.
#[inline]
pub fn check_stretch_goal_limit(env: &soroban_sdk::Env) -> Result<(), &'static str> {
    use soroban_sdk::Vec;
    let count: u32 = env
        .storage()
        .persistent()
        .get::<_, Vec<i128>>(&crate::DataKey::StretchGoals)
        .map(|v| v.len())
        .unwrap_or(0);
    validate_stretch_goal_capacity(count)
#[inline]
fn validate_next_count(
    current_count: u32,
    max_count: u32,
    error: &'static str,
) -> Result<(), &'static str> {
    if current_count >= max_count {
        return Err(error);
/// Assert that adding one more entry to the `Contributors` list is allowed.
///
/// Reads the current list length from persistent storage and compares it
/// against [`MAX_CONTRIBUTORS`].
///
/// @param env Soroban environment reference.
/// @return `Ok(())` when within limits, `Err(StateSizeError::ContributorLimitExceeded)` otherwise.
pub fn check_contributor_limit(env: &Env) -> Result<(), StateSizeError> {
    let contributors: Vec<soroban_sdk::Address> = env
        .storage()
        .persistent()
        .get(&DataKey::Contributors)
        .unwrap_or_else(|| Vec::new(env));

    if contributors.len() >= MAX_CONTRIBUTORS {
        return Err(StateSizeError::ContributorLimitExceeded);
    }
    Ok(())
}

/// Validates the campaign title length.
///
/// @param title Proposed title string.
/// @return `Ok(())` when `title.len() <= MAX_TITLE_LENGTH`.
pub fn validate_title(title: &String) -> Result<(), &'static str> {
    validate_string_length(
        title,
        MAX_TITLE_LENGTH,
        "title exceeds MAX_TITLE_LENGTH bytes",
    )
}

/// Validates the campaign description length.
///
/// @param description Proposed description string.
/// @return `Ok(())` when `description.len() <= MAX_DESCRIPTION_LENGTH`.
pub fn validate_description(description: &String) -> Result<(), &'static str> {
    validate_string_length(
        description,
        MAX_DESCRIPTION_LENGTH,
        "description exceeds MAX_DESCRIPTION_LENGTH bytes",
    )
}

/// Validates the social-links field length.
///
/// @param socials Proposed social-links string.
/// @return `Ok(())` when `socials.len() <= MAX_SOCIAL_LINKS_LENGTH`.
pub fn validate_social_links(socials: &String) -> Result<(), &'static str> {
    validate_string_length(
        socials,
        MAX_SOCIAL_LINKS_LENGTH,
        "social links exceed MAX_SOCIAL_LINKS_LENGTH bytes",
    )
}

/// Validates the optional bonus-goal description length.
///
/// @param description Proposed bonus-goal description.
/// @return `Ok(())` when the value fits within the configured bound.
pub fn validate_bonus_goal_description(description: &String) -> Result<(), &'static str> {
    validate_string_length(
        description,
        MAX_BONUS_GOAL_DESCRIPTION_LENGTH,
        "bonus goal description exceeds MAX_BONUS_GOAL_DESCRIPTION_LENGTH bytes",
    )
}

/// Validates a roadmap item description length.
///
/// @param description Proposed roadmap text.
/// @return `Ok(())` when the roadmap text is within the configured limit.
pub fn validate_roadmap_description(description: &String) -> Result<(), &'static str> {
    validate_string_length(
        description,
        MAX_ROADMAP_DESCRIPTION_LENGTH,
        "roadmap description exceeds MAX_ROADMAP_DESCRIPTION_LENGTH bytes",
    )
}

/// Validates the combined metadata footprint.
///
/// @param title_length Campaign title length in bytes.
/// @param description_length Campaign description length in bytes.
/// @param socials_length Social-links field length in bytes.
/// @return `Ok(())` when the total fits within `MAX_METADATA_TOTAL_LENGTH`.
pub fn validate_metadata_total_length(
    title_length: u32,
    description_length: u32,
    socials_length: u32,
) -> Result<(), &'static str> {
    let total = title_length
        .checked_add(description_length)
        .and_then(|value| value.checked_add(socials_length))
        .unwrap_or(u32::MAX);

    if total > MAX_METADATA_TOTAL_LENGTH {
        return Err("metadata exceeds MAX_METADATA_TOTAL_LENGTH bytes");
    }

    Ok(())
}

/// Validates that a new contributor can be added to the indexed contributor list.
///
/// @param current_count Current number of indexed contributors.
/// @return `Ok(())` when `current_count < MAX_CONTRIBUTORS`.
pub fn validate_contributor_capacity(current_count: u32) -> Result<(), &'static str> {
    validate_next_count(
        current_count,
        MAX_CONTRIBUTORS,
        "contributors exceed MAX_CONTRIBUTORS",
    )
}

/// Validates that a new pledger can be added to the indexed pledger list.
///
/// @param current_count Current number of indexed pledgers.
/// @return `Ok(())` when `current_count < MAX_PLEDGERS`.
pub fn validate_pledger_capacity(current_count: u32) -> Result<(), &'static str> {
    validate_next_count(current_count, MAX_PLEDGERS, "pledgers exceed MAX_PLEDGERS")
}

/// Validates that a new roadmap item can be appended.
///
/// @param current_count Current number of roadmap items.
/// @return `Ok(())` when `current_count < MAX_ROADMAP_ITEMS`.
pub fn validate_roadmap_capacity(current_count: u32) -> Result<(), &'static str> {
    validate_next_count(
        current_count,
        MAX_ROADMAP_ITEMS,
        "roadmap exceeds MAX_ROADMAP_ITEMS",
    )
}

/// Validates that a new stretch goal can be appended.
///
/// @param current_count Current number of stretch goals.
/// @return `Ok(())` when `current_count < MAX_STRETCH_GOALS`.
pub fn validate_stretch_goal_capacity(current_count: u32) -> Result<(), &'static str> {
    validate_next_count(
        current_count,
        MAX_STRETCH_GOALS,
        "stretch goals exceed MAX_STRETCH_GOALS",
    )
/// Assert that adding one more entry to the `Pledgers` list is allowed.
///
/// @param env Soroban environment reference.
/// @return `Ok(())` when within limits, `Err(StateSizeError::ContributorLimitExceeded)` otherwise.
pub fn check_pledger_limit(env: &Env) -> Result<(), StateSizeError> {
    let pledgers: Vec<soroban_sdk::Address> = env
        .storage()
        .persistent()
        .get(&DataKey::Pledgers)
        .unwrap_or_else(|| Vec::new(env));

    if pledgers.len() >= MAX_PLEDGERS {
        return Err(StateSizeError::ContributorLimitExceeded);
    }
    Ok(())
}

/// Assert that adding one more item to the `Roadmap` list is allowed.
///
/// @param env Soroban environment reference.
/// @return `Ok(())` when within limits, `Err(StateSizeError::RoadmapLimitExceeded)` otherwise.
pub fn check_roadmap_limit(env: &Env) -> Result<(), StateSizeError> {
    let roadmap: Vec<crate::RoadmapItem> = env
        .storage()
        .instance()
        .get(&DataKey::Roadmap)
        .unwrap_or_else(|| Vec::new(env));

    if roadmap.len() >= MAX_ROADMAP_ITEMS {
        return Err(StateSizeError::RoadmapLimitExceeded);
    }
    Ok(())
}

/// Assert that adding one more stretch goal is allowed.
///
/// @param env Soroban environment reference.
/// @return `Ok(())` when within limits, `Err(StateSizeError::StretchGoalLimitExceeded)` otherwise.
pub fn check_stretch_goal_limit(env: &Env) -> Result<(), StateSizeError> {
    let goals: Vec<i128> = env
        .storage()
        .instance()
        .get(&DataKey::StretchGoals)
        .unwrap_or_else(|| Vec::new(env));

    if goals.len() >= MAX_STRETCH_GOALS {
        return Err(StateSizeError::StretchGoalLimitExceeded);
    }
    Ok(())
}

// ── Named limits (used by lib.rs validate_* calls) ───────────────────────────

/// Maximum byte length of a campaign title.
pub const MAX_TITLE_LENGTH: u32 = MAX_STRING_LEN;
/// Maximum byte length of a campaign description.
pub const MAX_DESCRIPTION_LENGTH: u32 = MAX_STRING_LEN;
/// Maximum byte length of social links.
pub const MAX_SOCIAL_LINKS_LENGTH: u32 = MAX_STRING_LEN;
/// Maximum byte length of a roadmap item description.
pub const MAX_ROADMAP_DESCRIPTION_LENGTH: u32 = MAX_STRING_LEN;
/// Maximum byte length of a bonus-goal description.
pub const MAX_BONUS_GOAL_DESCRIPTION_LENGTH: u32 = MAX_STRING_LEN;
/// Maximum number of pledgers (alias of MAX_CONTRIBUTORS).
pub const MAX_PLEDGERS: u32 = MAX_CONTRIBUTORS;
/// Maximum combined byte length of all metadata fields.
pub const MAX_METADATA_TOTAL_LENGTH: u32 = MAX_STRING_LEN * 3;

// ── validate_* wrappers ───────────────────────────────────────────────────────

/// Validate that a title string is within bounds.
pub fn validate_title(s: &String) -> Result<(), StateSizeError> {
    check_string_len(s)
}

/// Validate that a description string is within bounds.
pub fn validate_description(s: &String) -> Result<(), StateSizeError> {
    check_string_len(s)
}

/// Validate that a social-links string is within bounds.
pub fn validate_social_links(s: &String) -> Result<(), StateSizeError> {
    check_string_len(s)
}

/// Validate that a roadmap item description is within bounds.
pub fn validate_roadmap_description(s: &String) -> Result<(), StateSizeError> {
    check_string_len(s)
}

/// Validate that a bonus-goal description is within bounds.
pub fn validate_bonus_goal_description(s: &String) -> Result<(), StateSizeError> {
    check_string_len(s)
}

/// Validate combined metadata length does not exceed [`MAX_METADATA_TOTAL_LENGTH`].
pub fn validate_metadata_total_length(
    title_len: u32,
    description_len: u32,
    socials_len: u32,
) -> Result<(), StateSizeError> {
    let total = title_len
        .saturating_add(description_len)
        .saturating_add(socials_len);
    if total > MAX_METADATA_TOTAL_LENGTH {
        return Err(StateSizeError::StringTooLong);
    }
    Ok(())
}

/// Validate that the contributor list has not reached capacity.
///
/// @param current_len Current length of the contributors list.
pub fn validate_contributor_capacity(current_len: u32) -> Result<(), StateSizeError> {
    if current_len >= MAX_CONTRIBUTORS {
        return Err(StateSizeError::ContributorLimitExceeded);
    }
    Ok(())
}

/// Validate that the pledger list has not reached capacity.
///
/// @param current_len Current length of the pledgers list.
pub fn validate_pledger_capacity(current_len: u32) -> Result<(), StateSizeError> {
    if current_len >= MAX_PLEDGERS {
        return Err(StateSizeError::ContributorLimitExceeded);
    }
    Ok(())
}

/// Validate that the roadmap list has not reached capacity.
///
/// @param current_len Current length of the roadmap list.
pub fn validate_roadmap_capacity(current_len: u32) -> Result<(), StateSizeError> {
    if current_len >= MAX_ROADMAP_ITEMS {
        return Err(StateSizeError::RoadmapLimitExceeded);
    }
    Ok(())
}

/// Validate that the stretch-goals list has not reached capacity.
///
/// @param current_len Current length of the stretch-goals list.
pub fn validate_stretch_goal_capacity(current_len: u32) -> Result<(), StateSizeError> {
    if current_len >= MAX_STRETCH_GOALS {
        return Err(StateSizeError::StretchGoalLimitExceeded);
    }
    Ok(())
}
