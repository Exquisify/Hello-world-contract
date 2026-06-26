//! # Prediction Oracle Verifier
//!
//! A Soroban smart contract that resolves on-chain predictions against
//! off-chain price data supplied by whitelisted oracle addresses.
//!
//! ## Roles
//! - **Admin** – set at `init`, can whitelist/revoke oracle addresses.
//! - **Oracle** – whitelisted address that calls `resolve_prediction` with
//!   real-world price data after a prediction's deadline has passed.
//! - **Anyone** – can read resolutions via `get_resolution`.
//!
//! ## Resolution logic
//! `actual_price >= target_price` → `ResolutionResult::Correct`
//! `actual_price <  target_price` → `ResolutionResult::Incorrect`

#![no_std]

#[cfg(test)]
extern crate std;

pub mod errors;
pub mod storage;
pub mod types;

#[cfg(test)]
mod tests;

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contracttype, panic_with_error,
    symbol_short, Address, Env, String, Vec,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdeaData {
    pub id: i128,
    pub author: Address,
    pub title: String,
    pub body: String,
    pub category: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub deleted: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VoteDirection {
    Up,
    Down,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteRecord {
    pub direction: VoteDirection,
    pub weight: i32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteCount {
    pub upvotes: i32,
    pub downvotes: i32,
    pub net_score: i32,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    NextIdeaId,
    Idea(i128),
    AuthorIdeas(Address),
    CategoryIdeas(String),
    Vote(i128, Address),
    VoteCount(i128),
    ReputationContract,
}

#[contractclient(name = "ReputationContractClient")]
pub trait ReputationContract {
    fn reputation_score(env: Env, user: Address) -> i32;
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum IdeaError {
    NotFound = 1,
    Unauthorized = 2,
    Deleted = 3,
    VoteAlreadyExists = 4,
    VoteNotFound = 5,
    ReputationContractNotConfigured = 6,
    InvalidVoteWeight = 7,
}

#[contract]
pub struct PredictionVerifier;

#[contractimpl]
impl IdeaContract {
    pub fn set_reputation_contract(env: Env, contract: Address) {
        env.storage()
            .persistent()
            .set(&DataKey::ReputationContract, &contract);
    }

    pub fn create_idea(
        env: Env,
        author: Address,
        title: String,
        body: String,
        category: String,
    ) -> i128 {
        author.require_auth();

        let idea_id = next_idea_id(&env);
        let now = env.ledger().timestamp();

        let idea = IdeaData {
            id: idea_id,
            author: author.clone(),
            title,
            body,
            category: category.clone(),
            created_at: now,
            updated_at: now,
            deleted: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Idea(idea_id), &idea);
        append_id(&env, DataKey::AuthorIdeas(author.clone()), idea_id);
        append_id(&env, DataKey::CategoryIdeas(category), idea_id);

        env.events()
            .publish((symbol_short!("Created"), author), idea_id);

    // ─── Oracle whitelist management (admin-only) ─────────────────────────────

    /// Whitelist an oracle address. Only the admin may call this.
    ///
    /// # Arguments
    /// * `caller`         – must be the admin
    /// * `oracle_address` – address to whitelist
    pub fn set_authorized_oracle(env: Env, caller: Address, oracle_address: Address) {
        require_admin(&env, &caller);
        add_oracle(&env, &oracle_address);

        env.events().publish(
            (symbol_short!("OracleAdd"), caller),
            oracle_address,
        );
    }

    /// Revoke an oracle from the whitelist. Only the admin may call this.
    ///
    /// # Arguments
    /// * `caller`         – must be the admin
    /// * `oracle_address` – address to revoke
    pub fn remove_authorized_oracle(env: Env, caller: Address, oracle_address: Address) {
        require_admin(&env, &caller);
        remove_oracle(&env, &oracle_address);

        env.events().publish(
            (symbol_short!("OracleRem"), caller),
            oracle_address,
        );
    }

    /// Return the current list of whitelisted oracle addresses.
    pub fn get_authorized_oracles(env: Env) -> Vec<Address> {
        get_authorized_oracles(&env)
    }

        let mut idea = get_idea(&env, idea_id);
        ensure_active_and_author(&env, &idea, &caller);

        idea.body = new_body;
        idea.updated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::Idea(idea_id), &idea);

        // 4. Determine outcome
        let result = if actual_price >= target_price {
            ResolutionResult::Correct
        } else {
            ResolutionResult::Incorrect
        };

        let correct = matches!(result, ResolutionResult::Correct);

        let mut idea = get_idea(&env, idea_id);
        ensure_active_and_author(&env, &idea, &caller);

        idea.deleted = true;
        idea.updated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::Idea(idea_id), &idea);

        // 5. Emit PredictionResolved event
        env.events().publish(
            (symbol_short!("PRSolved"), prediction_id),
            (correct, actual_price, oracle),
        );
    }

    pub fn vote(env: Env, idea_id: i128, voter: Address, direction: VoteDirection) {
        voter.require_auth();
        ensure_votable(&env, idea_id);

        let key = DataKey::Vote(idea_id, voter.clone());
        if env.storage().persistent().has(&key) {
            panic_with_error!(&env, IdeaError::VoteAlreadyExists);
        }

        let record = VoteRecord {
            direction: direction.clone(),
            weight: reputation_weight(&env, &voter),
        };

        apply_vote_delta(&env, idea_id, None, Some(&record));
        env.storage().persistent().set(&key, &record);

        env.events()
            .publish((symbol_short!("Voted"), idea_id, voter), (direction, record.weight));
    }

    pub fn change_vote(
        env: Env,
        idea_id: i128,
        voter: Address,
        new_direction: VoteDirection,
    ) {
        voter.require_auth();
        ensure_votable(&env, idea_id);

        let key = DataKey::Vote(idea_id, voter.clone());
        let existing: VoteRecord = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic_with_error!(&env, IdeaError::VoteNotFound));

        let updated = VoteRecord {
            direction: new_direction.clone(),
            weight: reputation_weight(&env, &voter),
        };

        apply_vote_delta(&env, idea_id, Some(&existing), Some(&updated));
        env.storage().persistent().set(&key, &updated);

        env.events().publish(
            (symbol_short!("Voted"), idea_id, voter),
            (new_direction, updated.weight),
        );
    }

    pub fn remove_vote(env: Env, idea_id: i128, voter: Address) {
        voter.require_auth();
        ensure_votable(&env, idea_id);

        let key = DataKey::Vote(idea_id, voter.clone());
        let existing: VoteRecord = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic_with_error!(&env, IdeaError::VoteNotFound));

        apply_vote_delta(&env, idea_id, Some(&existing), None);
        env.storage().persistent().remove(&key);

        env.events().publish(
            (symbol_short!("VoteRm"), idea_id, voter),
            (existing.direction, existing.weight),
        );
    }

    pub fn get_vote(env: Env, idea_id: i128, voter: Address) -> Option<VoteRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::Vote(idea_id, voter))
    }

    pub fn get_vote_count(env: Env, idea_id: i128) -> VoteCount {
        ensure_votable(&env, idea_id);

        env.storage()
            .persistent()
            .get(&DataKey::VoteCount(idea_id))
            .unwrap_or(VoteCount {
                upvotes: 0,
                downvotes: 0,
                net_score: 0,
            })
    }

    pub fn list_ideas_by_author(env: Env, author: Address) -> Vec<i128> {
        env.storage()
            .persistent()
            .get(&DataKey::AuthorIdeas(author))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Convenience: return `true` if a prediction has been resolved.
    pub fn is_resolved(env: Env, prediction_id: i128) -> bool {
        storage_get_resolution(&env, prediction_id).is_some()
    }
}

fn reputation_weight(env: &Env, voter: &Address) -> i32 {
    let contract: Address = env
        .storage()
        .persistent()
        .get(&DataKey::ReputationContract)
        .unwrap_or_else(|| panic_with_error!(env, IdeaError::ReputationContractNotConfigured));

    let score = ReputationContractClient::new(env, &contract).reputation_score(voter);
    let weight = score / 100;

    if weight <= 0 {
        panic_with_error!(env, IdeaError::InvalidVoteWeight);
    }

    weight
}

fn ensure_votable(env: &Env, idea_id: i128) {
    let idea = get_idea(env, idea_id);
    if idea.deleted {
        panic_with_error!(env, IdeaError::Deleted);
    }
}

fn ensure_active_and_author(env: &Env, idea: &IdeaData, caller: &Address) {
    if idea.deleted {
        panic_with_error!(env, IdeaError::Deleted);
    }
    if &idea.author != caller {
        panic_with_error!(env, IdeaError::Unauthorized);
    }
}

fn next_idea_id(env: &Env) -> i128 {
    let current = env
        .storage()
        .persistent()
        .get(&DataKey::NextIdeaId)
        .unwrap_or(0_i128);

    let next = current + 1;
    env.storage().persistent().set(&DataKey::NextIdeaId, &next);
    next
}

fn append_id(env: &Env, key: DataKey, idea_id: i128) {
    let mut ids = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));

    ids.push_back(idea_id);
    env.storage().persistent().set(&key, &ids);
}

fn get_idea(env: &Env, idea_id: i128) -> IdeaData {
    env.storage()
        .persistent()
        .get(&DataKey::Idea(idea_id))
        .unwrap_or_else(|| panic_with_error!(env, IdeaError::NotFound))
}

fn apply_vote_delta(
    env: &Env,
    idea_id: i128,
    old: Option<&VoteRecord>,
    new: Option<&VoteRecord>,
) {
    let mut counts: VoteCount = env
        .storage()
        .persistent()
        .get(&DataKey::VoteCount(idea_id))
        .unwrap_or(VoteCount {
            upvotes: 0,
            downvotes: 0,
            net_score: 0,
        });

    if let Some(v) = old {
        match v.direction {
            VoteDirection::Up => {
                counts.upvotes -= v.weight;
                counts.net_score -= v.weight;
            }
            VoteDirection::Down => {
                counts.downvotes -= v.weight;
                counts.net_score += v.weight;
            }
        }
    }

    if let Some(v) = new {
        match v.direction {
            VoteDirection::Up => {
                counts.upvotes += v.weight;
                counts.net_score += v.weight;
            }
            VoteDirection::Down => {
                counts.downvotes += v.weight;
                counts.net_score -= v.weight;
            }
        }
    }

    env.storage()
        .persistent()
        .set(&DataKey::VoteCount(idea_id), &counts);
}

#[cfg(test)]
mod tests;
