//! Error definitions for the Token Vault contract.

use soroban_sdk::contracterror;

/// Error codes for the token vault contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    /// Contract not initialized yet.
    NotInitialized = 1,
    /// Contract already initialized.
    AlreadyInitialized = 2,
    /// Invalid amount (must be > 0).
    InvalidAmount = 3,
    /// Insufficient balance.
    InsufficientBalance = 4,
    /// Insufficient allowance.
    InsufficientAllowance = 5,
    /// Vault is paused.
    VaultPaused = 6,
    /// Unauthorized access.
    Unauthorized = 7,
    /// Token not found.
    TokenNotFound = 8,
    /// Transfer failed.
    TransferFailed = 9,
}
