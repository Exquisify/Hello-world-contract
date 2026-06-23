use soroban_sdk::{Address, Env, Vec};

use crate::errors::VerifierError;
use crate::types::{PredictionData, Resolution, VerifierKey};

// ─── Admin ────────────────────────────────────────────────────────────────────

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&VerifierKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&VerifierKey::Admin, admin);
}

pub fn require_admin(env: &Env, caller: &Address) {
    let admin: Address = env
        .storage()
        .instance()
        .get(&VerifierKey::Admin)
        .unwrap_or_else(|| panic_with_verifier_error(env, VerifierError::NotInitialised));
    if admin != *caller {
        panic_with_verifier_error(env, VerifierError::Unauthorized);
    }
    caller.require_auth();
}

// ─── Authorized oracles ───────────────────────────────────────────────────────

pub fn get_authorized_oracles(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&VerifierKey::AuthorizedOracles)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn is_oracle_authorized(env: &Env, oracle: &Address) -> bool {
    get_authorized_oracles(env).contains(oracle)
}

pub fn add_oracle(env: &Env, oracle: &Address) {
    let mut oracles = get_authorized_oracles(env);
    if !oracles.contains(oracle) {
        oracles.push_back(oracle.clone());
        env.storage()
            .persistent()
            .set(&VerifierKey::AuthorizedOracles, &oracles);
    }
}

pub fn remove_oracle(env: &Env, oracle: &Address) {
    let oracles = get_authorized_oracles(env);
    let mut updated: Vec<Address> = Vec::new(env);
    for o in oracles.iter() {
        if o != *oracle {
            updated.push_back(o);
        }
    }
    env.storage()
        .persistent()
        .set(&VerifierKey::AuthorizedOracles, &updated);
}

// ─── Resolutions ─────────────────────────────────────────────────────────────

pub fn get_resolution(env: &Env, prediction_id: i128) -> Option<Resolution> {
    env.storage()
        .persistent()
        .get(&VerifierKey::Resolution(prediction_id))
}

pub fn save_resolution(env: &Env, resolution: &Resolution) {
    env.storage()
        .persistent()
        .set(&VerifierKey::Resolution(resolution.prediction_id), resolution);
}

// ─── Prediction cross-call stub ───────────────────────────────────────────────
//
// In production this would invoke the PredictionContract via a cross-contract
// call client. For now we expose a helper that reads the data from shared
// persistent storage when both contracts are deployed together, or panics with
// a clear error during standalone verifier testing.
//
// Swap this function body for a real cross-contract client once the
// PredictionContract interface is stable.

pub fn get_prediction(env: &Env, prediction_id: i128) -> PredictionData {
    use crate::types::VerifierKey;
    // Key mirrors what PredictionContract writes under DataKey::Prediction(id).
    // Both contracts share persistent storage in Soroban when same Wasm/footprint
    // is used; replace with a client call if deployed separately.
    env.storage()
        .persistent()
        .get(&VerifierKey::Resolution(prediction_id)) // placeholder — see note above
        .map(|_r: Resolution| {
            // If a resolution already exists the caller will catch AlreadyResolved
            // before we reach here in normal flow.
            panic_with_verifier_error(env, VerifierError::AlreadyResolved)
        })
        .unwrap_or_else(|| {
            // Real implementation: call PredictionContract.get_prediction(prediction_id)
            panic_with_verifier_error(env, VerifierError::PredictionNotFound)
        })
}

// ─── Internal panic helper ───────────────────────────────────────────────────

#[inline(always)]
pub fn panic_with_verifier_error(env: &Env, err: VerifierError) -> ! {
    soroban_sdk::panic_with_error!(env, err)
}