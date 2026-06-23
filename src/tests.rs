//! Integration tests for the PredictionVerifier contract.
//!
//! We use Soroban's test environment which gives us:
//!  - `Env::default()` — a fresh ledger with controllable timestamps
//!  - `Address::generate(&env)` — deterministic mock addresses
//!  - The full contract call stack without deploying to a network

#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{
    errors::VerifierError,
    types::{Resolution, ResolutionResult},
    PredictionVerifier, PredictionVerifierClient,
};

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Seed data for a typical prediction scenario.
struct TestContext {
    env: Env,
    admin: Address,
    oracle: Address,
    rogue: Address,
    client: PredictionVerifierClient<'static>,
    prediction_id: i128,
    target_price: u128,
    deadline: u64,
}

impl TestContext {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        // Deploy the verifier contract
        let contract_id = env.register_contract(None, PredictionVerifier);
        // SAFETY: we own env and the lifetime here matches test scope
        let client: PredictionVerifierClient<'static> =
            unsafe { core::mem::transmute(PredictionVerifierClient::new(&env, &contract_id)) };

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let rogue = Address::generate(&env);

        // Initialise — set deadline in the future (ledger starts at 0)
        client.init(&admin);
        client.set_authorized_oracle(&admin, &oracle);

        Self {
            env,
            admin,
            oracle,
            rogue,
            client,
            prediction_id: 42,
            target_price: 100_000_u128, // e.g. $100,000 in micro-units
            deadline: 1_000,
        }
    }

    /// Advance the mock ledger past the deadline.
    fn advance_past_deadline(&self) {
        self.env.ledger().with_mut(|l| {
            l.timestamp = self.deadline + 1;
        });
    }

    /// Helper: resolve with default values (actual >= target → Correct).
    fn resolve_correct(&self) -> Resolution {
        self.advance_past_deadline();
        self.client.resolve_prediction(
            &self.oracle,
            &self.prediction_id,
            &self.target_price,       // actual == target → Correct
            &self.target_price,
            &self.deadline,
            &(self.deadline + 1),
        );
        self.client
            .get_resolution(&self.prediction_id)
            .expect("resolution must exist after resolve_prediction")
    }
}

// ─── Init ────────────────────────────────────────────────────────────────────

#[test]
fn test_init_sets_admin() {
    let ctx = TestContext::new();
    assert_eq!(ctx.client.get_admin(), Some(ctx.admin));
}

#[test]
#[should_panic(expected = "AlreadyInitialised")]
fn test_init_cannot_be_called_twice() {
    let ctx = TestContext::new();
    ctx.client.init(&ctx.admin); // second call must panic
}

// ─── Oracle whitelist ─────────────────────────────────────────────────────────

#[test]
fn test_set_authorized_oracle_adds_to_list() {
    let ctx = TestContext::new();
    let oracles = ctx.client.get_authorized_oracles();
    assert!(oracles.contains(&ctx.oracle));
}

#[test]
fn test_remove_authorized_oracle_removes_from_list() {
    let ctx = TestContext::new();
    ctx.client.remove_authorized_oracle(&ctx.admin, &ctx.oracle);
    let oracles = ctx.client.get_authorized_oracles();
    assert!(!oracles.contains(&ctx.oracle));
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_non_admin_cannot_add_oracle() {
    let ctx = TestContext::new();
    ctx.client
        .set_authorized_oracle(&ctx.rogue, &ctx.rogue);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_non_admin_cannot_remove_oracle() {
    let ctx = TestContext::new();
    ctx.client
        .remove_authorized_oracle(&ctx.rogue, &ctx.oracle);
}

#[test]
fn test_multiple_oracles_can_be_whitelisted() {
    let ctx = TestContext::new();
    let oracle2 = Address::generate(&ctx.env);
    let oracle3 = Address::generate(&ctx.env);

    ctx.client.set_authorized_oracle(&ctx.admin, &oracle2);
    ctx.client.set_authorized_oracle(&ctx.admin, &oracle3);

    let list = ctx.client.get_authorized_oracles();
    assert_eq!(list.len(), 3);
}

// ─── Resolution: happy paths ──────────────────────────────────────────────────

#[test]
fn test_resolve_prediction_correct_when_actual_gte_target() {
    let ctx = TestContext::new();
    let resolution = ctx.resolve_correct();

    assert_eq!(resolution.result, ResolutionResult::Correct);
    assert_eq!(resolution.prediction_id, ctx.prediction_id);
    assert_eq!(resolution.actual_price, ctx.target_price);
    assert_eq!(resolution.oracle, ctx.oracle);
}

#[test]
fn test_resolve_prediction_correct_when_actual_exceeds_target() {
    let ctx = TestContext::new();
    ctx.advance_past_deadline();

    ctx.client.resolve_prediction(
        &ctx.oracle,
        &ctx.prediction_id,
        &(ctx.target_price + 50_000),
        &ctx.target_price,
        &ctx.deadline,
        &(ctx.deadline + 1),
    );

    let res = ctx
        .client
        .get_resolution(&ctx.prediction_id)
        .unwrap();
    assert_eq!(res.result, ResolutionResult::Correct);
}

#[test]
fn test_resolve_prediction_incorrect_when_actual_below_target() {
    let ctx = TestContext::new();
    ctx.advance_past_deadline();

    ctx.client.resolve_prediction(
        &ctx.oracle,
        &ctx.prediction_id,
        &(ctx.target_price - 1),
        &ctx.target_price,
        &ctx.deadline,
        &(ctx.deadline + 1),
    );

    let res = ctx
        .client
        .get_resolution(&ctx.prediction_id)
        .unwrap();
    assert_eq!(res.result, ResolutionResult::Incorrect);
}

#[test]
fn test_resolution_stores_all_fields_correctly() {
    let ctx = TestContext::new();
    ctx.advance_past_deadline();

    let actual_price = 98_765_u128;
    let ts = ctx.deadline + 5;

    ctx.client.resolve_prediction(
        &ctx.oracle,
        &ctx.prediction_id,
        &actual_price,
        &ctx.target_price,
        &ctx.deadline,
        &ts,
    );

    let res = ctx
        .client
        .get_resolution(&ctx.prediction_id)
        .unwrap();

    assert_eq!(res.prediction_id, ctx.prediction_id);
    assert_eq!(res.actual_price, actual_price);
    assert_eq!(res.oracle, ctx.oracle);
    assert_eq!(res.resolution_timestamp, ts);
    assert_eq!(res.result, ResolutionResult::Incorrect);
}

// ─── Resolution: error guards ─────────────────────────────────────────────────

#[test]
#[should_panic(expected = "OracleNotAuthorized")]
fn test_resolve_panics_when_oracle_not_whitelisted() {
    let ctx = TestContext::new();
    ctx.advance_past_deadline();

    ctx.client.resolve_prediction(
        &ctx.rogue,
        &ctx.prediction_id,
        &ctx.target_price,
        &ctx.target_price,
        &ctx.deadline,
        &(ctx.deadline + 1),
    );
}

#[test]
#[should_panic(expected = "DeadlineNotReached")]
fn test_resolve_panics_before_deadline() {
    let ctx = TestContext::new();
    // Ledger timestamp starts at 0, deadline is 1_000 — no advance needed

    ctx.client.resolve_prediction(
        &ctx.oracle,
        &ctx.prediction_id,
        &ctx.target_price,
        &ctx.target_price,
        &ctx.deadline,
        &ctx.deadline,
    );
}

#[test]
#[should_panic(expected = "AlreadyResolved")]
fn test_resolve_panics_if_already_resolved() {
    let ctx = TestContext::new();
    ctx.advance_past_deadline();

    // First resolution — ok
    ctx.client.resolve_prediction(
        &ctx.oracle,
        &ctx.prediction_id,
        &ctx.target_price,
        &ctx.target_price,
        &ctx.deadline,
        &(ctx.deadline + 1),
    );

    // Second resolution on same prediction — must panic
    ctx.client.resolve_prediction(
        &ctx.oracle,
        &ctx.prediction_id,
        &ctx.target_price,
        &ctx.target_price,
        &ctx.deadline,
        &(ctx.deadline + 2),
    );
}

#[test]
#[should_panic(expected = "OracleNotAuthorized")]
fn test_revoked_oracle_cannot_resolve() {
    let ctx = TestContext::new();
    ctx.advance_past_deadline();

    // Revoke the oracle first
    ctx.client
        .remove_authorized_oracle(&ctx.admin, &ctx.oracle);

    // Now oracle tries to resolve — must be rejected
    ctx.client.resolve_prediction(
        &ctx.oracle,
        &ctx.prediction_id,
        &ctx.target_price,
        &ctx.target_price,
        &ctx.deadline,
        &(ctx.deadline + 1),
    );
}

// ─── Queries ─────────────────────────────────────────────────────────────────

#[test]
fn test_get_resolution_returns_none_before_resolve() {
    let ctx = TestContext::new();
    assert_eq!(ctx.client.get_resolution(&ctx.prediction_id), None);
}

#[test]
fn test_is_resolved_returns_false_before_resolve() {
    let ctx = TestContext::new();
    assert!(!ctx.client.is_resolved(&ctx.prediction_id));
}

#[test]
fn test_is_resolved_returns_true_after_resolve() {
    let ctx = TestContext::new();
    ctx.resolve_correct();
    assert!(ctx.client.is_resolved(&ctx.prediction_id));
}

#[test]
fn test_independent_predictions_are_isolated() {
    let ctx = TestContext::new();
    ctx.advance_past_deadline();

    let pid_a: i128 = 1;
    let pid_b: i128 = 2;

    ctx.client.resolve_prediction(
        &ctx.oracle,
        &pid_a,
        &ctx.target_price,
        &ctx.target_price,
        &ctx.deadline,
        &(ctx.deadline + 1),
    );

    assert!(ctx.client.is_resolved(&pid_a));
    assert!(!ctx.client.is_resolved(&pid_b));

    let res_a = ctx.client.get_resolution(&pid_a).unwrap();
    assert_eq!(res_a.prediction_id, pid_a);
}

// ─── Mock oracle round-trip ───────────────────────────────────────────────────

/// Full scenario: admin onboards oracle, oracle resolves two predictions
/// with opposite outcomes, verify both stored correctly.
#[test]
fn test_mock_oracle_full_roundtrip() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PredictionVerifier);
    let client = PredictionVerifierClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let mock_oracle = Address::generate(&env); // ← the "mock oracle" from the AC

    client.init(&admin);
    client.set_authorized_oracle(&admin, &mock_oracle);

    let deadline = 500_u64;
    let target = 200_u128;

    env.ledger().with_mut(|l| l.timestamp = deadline + 10);

    // Prediction 10: price went up → Correct
    client.resolve_prediction(&mock_oracle, &10_i128, &(target + 1), &target, &deadline, &(deadline + 10));
    // Prediction 11: price fell → Incorrect
    client.resolve_prediction(&mock_oracle, &11_i128, &(target - 1), &target, &deadline, &(deadline + 10));

    assert_eq!(
        client.get_resolution(&10).unwrap().result,
        ResolutionResult::Correct
    );
    assert_eq!(
        client.get_resolution(&11).unwrap().result,
        ResolutionResult::Incorrect
    );

    // Revoke oracle — future calls must fail
    client.remove_authorized_oracle(&admin, &mock_oracle);
    assert!(!client.get_authorized_oracles().contains(&mock_oracle));
}