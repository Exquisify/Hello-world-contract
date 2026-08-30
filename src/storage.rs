//! Storage module for the Token Vault contract.
//! Handles all storage operations with proper key management.

use soroban_sdk::{contracttype, Address, Env};
use crate::types::{Allowance, Balance, VaultConfig};
use crate::errors::Error;

/// Storage keys for the vault contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Vault initialized flag
    Initialized,
    /// Admin address
    Admin,
    /// Token address
    Token,
    /// Vault configuration
    Config,
    /// User balance
    Balance(Address),
    /// Allowance from owner to spender
    Allowance(Address, Address),
    /// Vault paused flag
    Paused,
    /// Total deposits
    TotalDeposits,
}

/// Storage helper functions.
pub struct Storage;

impl Storage {
    // ─── Initialization ──────────────────────────────────────────────────────

    /// Check if the vault is initialized.
    pub fn is_initialized(env: &Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    /// Set the vault as initialized.
    pub fn set_initialized(env: &Env) {
        env.storage().instance().set(&DataKey::Initialized, &true);
    }

    // ─── Admin ───────────────────────────────────────────────────────────────

    /// Get the admin address.
    pub fn get_admin(env: &Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }

    /// Set the admin address.
    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&DataKey::Admin, admin);
    }

    // ─── Token ──────────────────────────────────────────────────────────────

    /// Get the token address.
    pub fn get_token(env: &Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Token)
            .ok_or(Error::NotInitialized)
    }

    /// Set the token address.
    pub fn set_token(env: &Env, token: &Address) {
        env.storage().instance().set(&DataKey::Token, token);
    }

    // ─── Config ─────────────────────────────────────────────────────────────

    /// Get the vault configuration.
    pub fn get_config(env: &Env) -> Result<VaultConfig, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)
    }

    /// Set the vault configuration.
    pub fn set_config(env: &Env, config: &VaultConfig) {
        env.storage().instance().set(&DataKey::Config, config);
    }

    // ─── Pause ──────────────────────────────────────────────────────────────

    /// Check if the vault is paused.
    pub fn is_paused(env: &Env) -> bool {
        env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
    }

    /// Pause the vault.
    pub fn pause_vault(env: &Env) {
        env.storage().instance().set(&DataKey::Paused, &true);
    }

    /// Unpause the vault.
    pub fn unpause_vault(env: &Env) {
        env.storage().instance().set(&DataKey::Paused, &false);
    }

    // ─── Balance ────────────────────────────────────────────────────────────

    /// Get a user's balance.
    pub fn get_balance(env: &Env, user: &Address) -> Balance {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(user.clone()))
            .unwrap_or(Balance {
                amount: 0,
                last_updated: env.ledger().timestamp(),
            })
    }

    /// Set a user's balance.
    pub fn set_balance(env: &Env, user: &Address, balance: &Balance) {
        env.storage()
            .persistent()
            .set(&DataKey::Balance(user.clone()), balance);
    }

    // ─── Allowance ──────────────────────────────────────────────────────────

    /// Get the allowance from owner to spender.
    pub fn get_allowance(env: &Env, owner: &Address, spender: &Address) -> Allowance {
        env.storage()
            .persistent()
            .get(&DataKey::Allowance(owner.clone(), spender.clone()))
            .unwrap_or(Allowance {
                amount: 0,
                spender: spender.clone(),
            })
    }

    /// Set the allowance from owner to spender.
    pub fn set_allowance(env: &Env, owner: &Address, spender: &Address, allowance: &Allowance) {
        env.storage()
            .persistent()
            .set(&DataKey::Allowance(owner.clone(), spender.clone()), allowance);
    }

    // ─── Total Deposits ─────────────────────────────────────────────────────

    /// Get total deposits.
    pub fn get_total_deposits(env: &Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::TotalDeposits)
            .unwrap_or(0)
    }

    /// Add to total deposits.
    pub fn add_to_total_deposits(env: &Env, amount: i128) {
        let current = Self::get_total_deposits(env);
        env.storage()
            .persistent()
            .set(&DataKey::TotalDeposits, &(current + amount));
    }

    // ─── Events ─────────────────────────────────────────────────────────────

    /// Emit a deposit event.
    pub fn emit_deposit_event(env: &Env, user: &Address, amount: i128) {
        env.events()
            .publish(("deposit", user), amount);
    }

    /// Emit a withdraw event.
    pub fn emit_withdraw_event(env: &Env, user: &Address, amount: i128) {
        env.events()
            .publish(("withdraw", user), amount);
    }

    /// Emit an approval event.
    pub fn emit_approval_event(env: &Env, owner: &Address, spender: &Address, amount: i128) {
        env.events()
            .publish(("approval", owner, spender), amount);
    }
}
