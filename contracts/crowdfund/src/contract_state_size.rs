#![allow(missing_docs)]

use soroban_sdk::{Env, String, Vec};

use crate::DataKey;

// ── Limits ───────────────────────────────────────────────────────────────────

/// Maximum number of unique contributors tracked on-chain.
pub const MAX_CONTRIBUTORS: u32 = 1_000;

/// Maximum number of unique pledgers tracked on-chain.
pub const MAX_PLEDGERS: u32 = 1_000;

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
pub const MAX_BONUS_GOAL_DESCRIPTION_LENGTH: u32 = 500;
/// Maximum roadmap item description length in bytes.
pub const MAX_ROADMAP_DESCRIPTION_LENGTH: u32 = 500;
/// Maximum combined metadata budget (`title + description + socials`) in bytes.
pub const MAX_METADATA_TOTAL_LENGTH: u32 = 4_000;
/// Backward-compatible generic string limit used by legacy tests/helpers.
pub const MAX_STRING_LEN: u32 = 256;

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StateSizeError {
    ContributorLimitExceeded,
    RoadmapLimitExceeded,
    StretchGoalLimitExceeded,
    StringTooLong,
}

impl core::fmt::Display for StateSizeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StateSizeError::ContributorLimitExceeded => write!(f, "contributor limit exceeded"),
            StateSizeError::RoadmapLimitExceeded => write!(f, "roadmap limit exceeded"),
            StateSizeError::StretchGoalLimitExceeded => write!(f, "stretch goal limit exceeded"),
            StateSizeError::StringTooLong => write!(f, "string too long"),
        }
    }
}

// ── Validation helpers ────────────────────────────────────────────────────────

pub fn validate_title(title: &String) -> Result<(), &'static str> {
    if title.len() > MAX_TITLE_LENGTH {
        return Err("title exceeds MAX_TITLE_LENGTH bytes");
    }
    Ok(())
}

pub fn validate_description(value: &String) -> Result<(), &'static str> {
    if value.len() > MAX_DESCRIPTION_LENGTH {
        return Err("description exceeds MAX_DESCRIPTION_LENGTH bytes");
    }
    Ok(())
}

pub fn validate_social_links(value: &String) -> Result<(), &'static str> {
    if value.len() > MAX_SOCIAL_LINKS_LENGTH {
        return Err("social links exceed MAX_SOCIAL_LINKS_LENGTH bytes");
    }
    Ok(())
}

pub fn validate_bonus_goal_description(value: &String) -> Result<(), &'static str> {
    if value.len() > MAX_BONUS_GOAL_DESCRIPTION_LENGTH {
        return Err("bonus goal description exceeds MAX_BONUS_GOAL_DESCRIPTION_LENGTH bytes");
    }
    Ok(())
}

pub fn validate_roadmap_description(value: &String) -> Result<(), &'static str> {
    if value.len() > MAX_ROADMAP_DESCRIPTION_LENGTH {
        return Err("roadmap description exceeds MAX_ROADMAP_DESCRIPTION_LENGTH bytes");
    }
    Ok(())
}

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
        return Err("metadata exceeds MAX_METADATA_TOTAL_LENGTH bytes");
    }
    Ok(())
}

pub fn validate_contributor_capacity(len: u32) -> Result<(), &'static str> {
    if len >= MAX_CONTRIBUTORS {
        return Err("contributors exceed MAX_CONTRIBUTORS");
    }
    Ok(())
}

pub fn validate_pledger_capacity(len: u32) -> Result<(), &'static str> {
    if len >= MAX_PLEDGERS {
        return Err("pledgers exceed MAX_PLEDGERS");
    }
    Ok(())
}

pub fn validate_roadmap_capacity(len: u32) -> Result<(), &'static str> {
    if len >= MAX_ROADMAP_ITEMS {
        return Err("roadmap exceeds MAX_ROADMAP_ITEMS");
    }
    Ok(())
}

pub fn validate_stretch_goal_capacity(len: u32) -> Result<(), &'static str> {
    if len >= MAX_STRETCH_GOALS {
        return Err("stretch goals exceed MAX_STRETCH_GOALS");
    }
    Ok(())
}

pub fn check_string_len(s: &String) -> Result<(), StateSizeError> {
    if s.len() > MAX_STRING_LEN {
        return Err(StateSizeError::StringTooLong);
    }
    Ok(())
}

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
