use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VerifierError {
    /// Prediction not found in storage.
    PredictionNotFound = 1,
    /// Caller is not the contract admin.
    Unauthorized = 2,
    /// The supplied oracle address is not in the whitelist.
    OracleNotAuthorized = 3,
    /// The prediction deadline has not yet passed.
    DeadlineNotReached = 4,
    /// This prediction has already been resolved.
    AlreadyResolved = 5,
    /// Resolution record does not exist for this prediction.
    ResolutionNotFound = 6,
    /// Admin has not been initialised (call init first).
    NotInitialised = 7,
    /// Admin is already set; init cannot be called twice.
    AlreadyInitialised = 8,
}