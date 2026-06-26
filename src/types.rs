use soroban_sdk::{contracttype, Address};

/// The outcome of a resolved prediction.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolutionResult {
    /// actual_price >= target_price
    Correct,
    /// actual_price < target_price
    Incorrect,
}

/// Stored on-chain after a prediction is resolved by an oracle.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolution {
    pub prediction_id: i128,
    pub result: ResolutionResult,
    pub actual_price: u128,
    pub oracle: Address,
    pub resolution_timestamp: u64,
}

/// A prediction that can be targeted for resolution.
///
/// Fields must mirror whatever the outer PredictionContract stores.
/// We keep only what the verifier needs so this module stays decoupled.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredictionData {
    pub id: i128,
    pub author: Address,
    pub target_price: u128,
    pub deadline: u64,
    pub resolved: bool,
}

/// Storage key namespace for the verifier module.
#[contracttype]
#[derive(Clone)]
pub enum VerifierKey {
    /// Set<Address> — whitelisted oracle addresses (admin-controlled)
    AuthorizedOracles,
    /// Resolution record keyed by prediction id
    Resolution(i128),
    /// The contract admin (set at deploy time / via init)
    Admin,
}