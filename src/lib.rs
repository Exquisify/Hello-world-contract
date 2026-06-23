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
    contract, contractimpl, panic_with_error, symbol_short, Address, Env, Vec,
};

use errors::VerifierError;
use storage::{
    add_oracle, get_admin, get_authorized_oracles, get_resolution as storage_get_resolution,
    is_oracle_authorized, remove_oracle, require_admin, save_resolution, set_admin,
};
use types::{Resolution, ResolutionResult, VerifierKey};

#[contract]
pub struct PredictionVerifier;

#[contractimpl]
impl PredictionVerifier {
    // ─── Initialisation ──────────────────────────────────────────────────────

    /// Set the contract admin. Must be called once after deployment.
    /// Panics with `AlreadyInitialised` if called again.
    pub fn init(env: Env, admin: Address) {
        if get_admin(&env).is_some() {
            panic_with_error!(&env, VerifierError::AlreadyInitialised);
        }
        admin.require_auth();
        set_admin(&env, &admin);
    }

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

    // ─── Resolution ───────────────────────────────────────────────────────────

    /// Resolve a prediction against oracle-supplied price data.
    ///
    /// # Arguments
    /// * `oracle`                – must be a whitelisted oracle address
    /// * `prediction_id`         – id of the prediction to resolve
    /// * `actual_price`          – the real-world price at resolution time
    /// * `target_price`          – the prediction's original target price
    /// * `deadline`              – the prediction's deadline (unix timestamp)
    /// * `resolution_timestamp`  – when the oracle observed the price
    ///
    /// # Errors
    /// * `OracleNotAuthorized` – oracle not in whitelist
    /// * `DeadlineNotReached`  – prediction deadline has not yet passed
    /// * `AlreadyResolved`     – prediction was already resolved
    ///
    /// # Events
    /// Emits `PredictionResolved(prediction_id, correct: bool, actual_price, oracle)`
    pub fn resolve_prediction(
        env: Env,
        oracle: Address,
        prediction_id: i128,
        actual_price: u128,
        target_price: u128,
        deadline: u64,
        resolution_timestamp: u64,
    ) {
        // 1. Oracle must be whitelisted
        if !is_oracle_authorized(&env, &oracle) {
            panic_with_error!(&env, VerifierError::OracleNotAuthorized);
        }
        oracle.require_auth();

        // 2. Deadline must have passed
        let now = env.ledger().timestamp();
        if now < deadline {
            panic_with_error!(&env, VerifierError::DeadlineNotReached);
        }

        // 3. Guard: cannot re-resolve
        if storage_get_resolution(&env, prediction_id).is_some() {
            panic_with_error!(&env, VerifierError::AlreadyResolved);
        }

        // 4. Determine outcome
        let result = if actual_price >= target_price {
            ResolutionResult::Correct
        } else {
            ResolutionResult::Incorrect
        };

        let correct = matches!(result, ResolutionResult::Correct);

        let resolution = Resolution {
            prediction_id,
            result,
            actual_price,
            oracle: oracle.clone(),
            resolution_timestamp,
        };

        save_resolution(&env, &resolution);

        // 5. Emit PredictionResolved event
        env.events().publish(
            (symbol_short!("PRSolved"), prediction_id),
            (correct, actual_price, oracle),
        );
    }

    // ─── Queries ─────────────────────────────────────────────────────────────

    /// Return the resolution for a given prediction, or `None` if not yet resolved.
    pub fn get_resolution(env: Env, prediction_id: i128) -> Option<Resolution> {
        storage_get_resolution(&env, prediction_id)
    }

    /// Convenience: return `true` if a prediction has been resolved.
    pub fn is_resolved(env: Env, prediction_id: i128) -> bool {
        storage_get_resolution(&env, prediction_id).is_some()
    }

    /// Return the admin address, or `None` if not yet initialised.
    pub fn get_admin(env: Env) -> Option<Address> {
        get_admin(&env)
    }
}