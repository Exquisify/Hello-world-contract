//! Type definitions for the Token Vault contract.

use soroban_sdk::{Address, contracttype};

/// Vault configuration.
#[contracttype]
#[derive(Clone, Debug)]
pub struct VaultConfig {
    /// Whether the vault is paused
    pub paused: bool,
    /// Admin address
    pub owner: Address,
    /// Token contract address
    pub token: Address,
}

/// User balance.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Balance {
    /// Amount of tokens
    pub amount: i128,
    /// Last update timestamp
    pub last_updated: u64,
}

/// Allowance from owner to spender.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Allowance {
    /// Amount allowed to spend
    pub amount: i128,
    /// Spender address
    pub spender: Address,
}
