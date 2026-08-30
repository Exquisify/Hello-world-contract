//! Unit tests for the Token Vault contract.

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _},
        Address, Env,
    };
    use crate::{TokenVaultContract, TokenVaultContractClient};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let token = Address::generate(&env);

        let contract_id = env.register_contract(None, TokenVaultContract);
        let client = TokenVaultContractClient::new(&env, &contract_id);

        // Initialize
        assert!(client.try_initialize(&admin, &token).is_ok());

        // Verify config
        let config = client.get_config();
        assert_eq!(config.owner, admin);
        assert_eq!(config.token, token);
        assert_eq!(config.paused, false);
    }

    #[test]
    fn test_initialize_twice_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let token = Address::generate(&env);

        let contract_id = env.register_contract(None, TokenVaultContract);
        let client = TokenVaultContractClient::new(&env, &contract_id);

        assert!(client.try_initialize(&admin, &token).is_ok());
        assert!(client.try_initialize(&admin, &token).is_err());
    }

    #[test]
    fn test_deposit_zero_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let token = Address::generate(&env);

        let contract_id = env.register_contract(None, TokenVaultContract);
        let client = TokenVaultContractClient::new(&env, &contract_id);

        client.initialize(&admin, &token);

        // Deposit zero should fail
        assert!(client.try_deposit(&user, &0).is_err());
    }

    #[test]
    fn test_withdraw_zero_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let token = Address::generate(&env);

        let contract_id = env.register_contract(None, TokenVaultContract);
        let client = TokenVaultContractClient::new(&env, &contract_id);

        client.initialize(&admin, &token);

        // Withdraw zero should fail
        assert!(client.try_withdraw(&user, &0).is_err());
    }

    #[test]
    fn test_allowance() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let spender = Address::generate(&env);
        let token = Address::generate(&env);

        let contract_id = env.register_contract(None, TokenVaultContract);
        let client = TokenVaultContractClient::new(&env, &contract_id);

        client.initialize(&admin, &token);

        // Approve allowance
        assert!(client.try_approve(&owner, &spender, &500).is_ok());
        assert_eq!(client.allowance(&owner, &spender), 500);

        // Spend allowance (will fail due to insufficient balance, but tests the function)
        assert!(client.try_spend_allowance(&spender, &owner, &100).is_err());
    }

    #[test]
    fn test_approve_negative_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let spender = Address::generate(&env);
        let token = Address::generate(&env);

        let contract_id = env.register_contract(None, TokenVaultContract);
        let client = TokenVaultContractClient::new(&env, &contract_id);

        client.initialize(&admin, &token);

        // Approve negative amount should fail
        assert!(client.try_approve(&owner, &spender, &-100).is_err());
    }

    #[test]
    fn test_pause_and_unpause() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let token = Address::generate(&env);

        let contract_id = env.register_contract(None, TokenVaultContract);
        let client = TokenVaultContractClient::new(&env, &contract_id);

        client.initialize(&admin, &token);

        assert!(client.try_pause(&admin).is_ok());
        assert_eq!(client.is_paused(), true);

        assert!(client.try_unpause(&admin).is_ok());
        assert_eq!(client.is_paused(), false);
    }

    #[test]
    fn test_pause_only_admin() {
        let env = Env::default();
        
        // Create admin and attacker
        let admin = Address::generate(&env);
        let attacker = Address::generate(&env);
        let token = Address::generate(&env);

        let contract_id = env.register_contract(None, TokenVaultContract);
        let client = TokenVaultContractClient::new(&env, &contract_id);

        // Initialize with admin (needs auth)
        // We'll mock auth only for the initialization
        env.mock_all_auths();
        client.initialize(&admin, &token);
        // Revert to real auth mode by creating a new environment for the test
        // The contract uses admin.require_auth() in pause()
        // So we need to test that attacker cannot call pause
        
        // Test that attacker cannot pause
        // The contract should fail because attacker is not authorized
        // Since we're in test environment, we use try_pause which returns Result
        // and expect it to be an error
        assert!(client.try_pause(&attacker).is_err());
    }

    #[test]
    fn test_get_admin() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let token = Address::generate(&env);

        let contract_id = env.register_contract(None, TokenVaultContract);
        let client = TokenVaultContractClient::new(&env, &contract_id);

        client.initialize(&admin, &token);

        let retrieved_admin = client.get_admin();
        assert_eq!(retrieved_admin, admin);
    }

    #[test]
    fn test_get_config() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let token = Address::generate(&env);

        let contract_id = env.register_contract(None, TokenVaultContract);
        let client = TokenVaultContractClient::new(&env, &contract_id);

        client.initialize(&admin, &token);

        let config = client.get_config();
        assert_eq!(config.owner, admin);
        assert_eq!(config.token, token);
        assert_eq!(config.paused, false);
    }

    #[test]
    fn test_balance_returns_zero_for_new_user() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let token = Address::generate(&env);

        let contract_id = env.register_contract(None, TokenVaultContract);
        let client = TokenVaultContractClient::new(&env, &contract_id);

        client.initialize(&admin, &token);

        // New user balance should be 0
        assert_eq!(client.balance(&user), 0);
    }
}

// ─── Moderation Contract Tests ────────────────────────────────────────────────

#[cfg(test)]
mod moderation_tests {
    use crate::moderation::*;
    use soroban_sdk::{
        testutils::Address as _,
        Address, Env, String,
    };

    fn setup() -> (Env, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let moderator = Address::generate(&env);
        let user = Address::generate(&env);

        initialize_moderation(&env, admin.clone());
        add_moderator(&env, admin.clone(), moderator.clone());

        (env, admin, moderator, user)
    }

    fn text(env: &Env, s: &str) -> String {
        String::from_str(env, s)
    }

    // ─── Initialization tests ────────────────────────────────────────────────

    #[test]
    fn initialize_moderation_sets_admin_and_flag_count() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        initialize_moderation(&env, admin.clone());

        assert_eq!(get_flag_count(&env), 0);
    }

    // ─── Moderator management tests ──────────────────────────────────────────

    #[test]
    fn admin_can_add_moderator() {
        let (env, admin, moderator, _) = setup();

        assert!(is_moderator(&env, moderator));
    }

    #[test]
    fn admin_can_remove_moderator() {
        let (env, admin, moderator, _) = setup();

        remove_moderator(&env, admin.clone(), moderator.clone());

        assert!(!is_moderator(&env, moderator));
    }

    #[test]
    fn non_admin_cannot_add_moderator() {
        let (env, admin, moderator, user) = setup();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            add_moderator(&env, user.clone(), moderator.clone());
        }));

        assert!(result.is_err());
    }

    #[test]
    fn non_admin_cannot_remove_moderator() {
        let (env, admin, moderator, user) = setup();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            remove_moderator(&env, user.clone(), moderator.clone());
        }));

        assert!(result.is_err());
    }

    // ─── Flag submission tests ───────────────────────────────────────────────

    #[test]
    fn user_can_flag_content_with_reason_and_notes() {
        let (env, _, _, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Spam,
            Some(text(&env, "Multiple promotional messages")),
        );

        assert_eq!(flag_id, 1);
        assert_eq!(get_flag_count(&env), 1);

        let flag = get_flag(&env, flag_id).unwrap();
        assert_eq!(flag.flag_id, 1);
        assert_eq!(flag.status, FlagStatus::Pending);
        assert_eq!(flag.reason, FlagReason::Spam);
    }

    #[test]
    fn flag_increments_count() {
        let (env, _, _, user) = setup();

        for i in 0..5 {
            flag_content(
                &env,
                user.clone(),
                text(&env, &format!("content_{}", i)),
                FlagReason::Abusive,
                None,
            );
        }

        assert_eq!(get_flag_count(&env), 5);
    }

    #[test]
    fn flag_with_empty_content_id_fails() {
        let (env, _, _, user) = setup();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            flag_content(
                &env,
                user.clone(),
                text(&env, ""),
                FlagReason::PolicyViolation,
                None,
            );
        }));

        assert!(result.is_err());
    }

    #[test]
    fn flag_content_with_different_reasons() {
        let (env, _, _, user) = setup();

        let reasons = vec![
            FlagReason::Spam,
            FlagReason::Abusive,
            FlagReason::PolicyViolation,
            FlagReason::Other,
        ];

        for (i, reason) in reasons.iter().enumerate() {
            let flag_id = flag_content(
                &env,
                user.clone(),
                text(&env, &format!("content_{}", i)),
                reason.clone(),
                None,
            );

            let flag = get_flag(&env, flag_id).unwrap();
            assert_eq!(flag.reason, reason.clone());
        }
    }

    // ─── Flag review tests ──────────────────────────────────────────────────

    #[test]
    fn moderator_can_confirm_flag() {
        let (env, _, moderator, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Abusive,
            None,
        );

        review_flag(&env, moderator, flag_id, true);

        let flag = get_flag(&env, flag_id).unwrap();
        assert_eq!(flag.status, FlagStatus::Confirmed);
        assert!(flag.reviewed_by.is_some());
        assert!(flag.reviewed_ledger.is_some());
    }

    #[test]
    fn moderator_can_reject_flag() {
        let (env, _, moderator, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Spam,
            None,
        );

        review_flag(&env, moderator, flag_id, false);

        let flag = get_flag(&env, flag_id).unwrap();
        assert_eq!(flag.status, FlagStatus::Rejected);
    }

    #[test]
    fn non_moderator_cannot_review_flag() {
        let (env, _, _, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Abusive,
            None,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            review_flag(&env, user.clone(), flag_id, true);
        }));

        assert!(result.is_err());
    }

    #[test]
    fn cannot_review_non_pending_flag() {
        let (env, _, moderator, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Abusive,
            None,
        );

        review_flag(&env, moderator.clone(), flag_id, true);

        // Try to review again
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            review_flag(&env, moderator.clone(), flag_id, true);
        }));

        assert!(result.is_err());
    }

    #[test]
    fn review_nonexistent_flag_fails() {
        let (env, _, moderator, _) = setup();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            review_flag(&env, moderator, 999, true);
        }));

        assert!(result.is_err());
    }

    // ─── Appeal tests ───────────────────────────────────────────────────────

    #[test]
    fn content_owner_can_appeal_confirmed_flag() {
        let (env, _, moderator, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Abusive,
            None,
        );

        review_flag(&env, moderator, flag_id, true);

        appeal_flag(
            &env,
            user,
            flag_id,
            text(&env, "This content is not abusive"),
        );

        let flag = get_flag(&env, flag_id).unwrap();
        assert_eq!(flag.status, FlagStatus::Appealed);
        assert!(flag.appeal_notes.is_some());
    }

    #[test]
    fn cannot_appeal_pending_flag() {
        let (env, _, _, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Abusive,
            None,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            appeal_flag(&env, user, flag_id, text(&env, "appeal notes"));
        }));

        assert!(result.is_err());
    }

    #[test]
    fn cannot_appeal_rejected_flag() {
        let (env, _, moderator, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Spam,
            None,
        );

        review_flag(&env, moderator, flag_id, false);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            appeal_flag(&env, user, flag_id, text(&env, "appeal notes"));
        }));

        assert!(result.is_err());
    }

    // ─── Appeal resolution tests ────────────────────────────────────────────

    #[test]
    fn moderator_can_grant_appeal() {
        let (env, _, moderator, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Abusive,
            None,
        );

        review_flag(&env, moderator.clone(), flag_id, true);
        appeal_flag(&env, user, flag_id, text(&env, "appeal notes"));

        resolve_appeal(&env, moderator, flag_id, true);

        let flag = get_flag(&env, flag_id).unwrap();
        assert_eq!(flag.status, FlagStatus::AppealGranted);
    }

    #[test]
    fn moderator_can_deny_appeal() {
        let (env, _, moderator, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Abusive,
            None,
        );

        review_flag(&env, moderator.clone(), flag_id, true);
        appeal_flag(&env, user, flag_id, text(&env, "appeal notes"));

        resolve_appeal(&env, moderator, flag_id, false);

        let flag = get_flag(&env, flag_id).unwrap();
        assert_eq!(flag.status, FlagStatus::AppealDenied);
    }

    #[test]
    fn non_moderator_cannot_resolve_appeal() {
        let (env, _, moderator, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Abusive,
            None,
        );

        review_flag(&env, moderator, flag_id, true);
        appeal_flag(&env, user.clone(), flag_id, text(&env, "appeal notes"));

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            resolve_appeal(&env, user.clone(), flag_id, true);
        }));

        assert!(result.is_err());
    }

    #[test]
    fn cannot_resolve_non_appealed_flag() {
        let (env, _, moderator, _) = setup();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            resolve_appeal(&env, moderator, 999, true);
        }));

        assert!(result.is_err());
    }

    // ─── State machine enforcement tests ────────────────────────────────────

    #[test]
    fn flag_status_transitions_are_enforced() {
        let (env, _, moderator, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Abusive,
            None,
        );

        // Pending → Confirmed (valid)
        review_flag(&env, moderator.clone(), flag_id, true);
        assert_eq!(get_flag(&env, flag_id).unwrap().status, FlagStatus::Confirmed);

        // Confirmed → Appealed (valid)
        appeal_flag(&env, user, flag_id, text(&env, "appeal notes"));
        assert_eq!(get_flag(&env, flag_id).unwrap().status, FlagStatus::Appealed);

        // Appealed → AppealGranted (valid)
        resolve_appeal(&env, moderator, flag_id, true);
        assert_eq!(
            get_flag(&env, flag_id).unwrap().status,
            FlagStatus::AppealGranted
        );
    }

    // ─── Query tests ────────────────────────────────────────────────────────

    #[test]
    fn get_flag_returns_correct_data() {
        let (env, _, _, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_xyz"),
            FlagReason::PolicyViolation,
            Some(text(&env, "violates ToS section 5.2")),
        );

        let flag = get_flag(&env, flag_id).unwrap();

        assert_eq!(flag.flag_id, flag_id);
        assert_eq!(flag.content_id, text(&env, "content_xyz"));
        assert_eq!(flag.flagged_by, user);
        assert_eq!(flag.reason, FlagReason::PolicyViolation);
        assert_eq!(flag.notes, Some(text(&env, "violates ToS section 5.2")));
        assert_eq!(flag.status, FlagStatus::Pending);
    }

    #[test]
    fn get_nonexistent_flag_returns_none() {
        let (env, _, _, _) = setup();

        assert_eq!(get_flag(&env, 999), None);
    }

    #[test]
    fn get_flag_count_reflects_all_flags() {
        let (env, _, _, user) = setup();

        assert_eq!(get_flag_count(&env), 0);

        flag_content(
            &env,
            user.clone(),
            text(&env, "content_1"),
            FlagReason::Spam,
            None,
        );
        assert_eq!(get_flag_count(&env), 1);

        flag_content(
            &env,
            user.clone(),
            text(&env, "content_2"),
            FlagReason::Abusive,
            None,
        );
        assert_eq!(get_flag_count(&env), 2);
    }

    #[test]
    fn is_moderator_returns_correct_status() {
        let (env, _, moderator, user) = setup();

        assert!(is_moderator(&env, moderator));
        assert!(!is_moderator(&env, user));
    }

    // ─── Event emission tests ───────────────────────────────────────────────

    #[test]
    fn flag_content_emits_event() {
        let (env, _, _, user) = setup();

        flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Spam,
            None,
        );

        let events = env.events().all();
        assert!(events.len() > 0);
    }

    #[test]
    fn review_flag_emits_event() {
        let (env, _, moderator, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Abusive,
            None,
        );

        let initial_event_count = env.events().all().len();
        review_flag(&env, moderator, flag_id, true);
        let final_event_count = env.events().all().len();

        assert!(final_event_count > initial_event_count);
    }

    #[test]
    fn appeal_flag_emits_event() {
        let (env, _, moderator, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Abusive,
            None,
        );

        review_flag(&env, moderator, flag_id, true);

        let initial_event_count = env.events().all().len();
        appeal_flag(&env, user, flag_id, text(&env, "appeal notes"));
        let final_event_count = env.events().all().len();

        assert!(final_event_count > initial_event_count);
    }

    #[test]
    fn resolve_appeal_emits_event() {
        let (env, _, moderator, user) = setup();

        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Abusive,
            None,
        );

        review_flag(&env, moderator.clone(), flag_id, true);
        appeal_flag(&env, user, flag_id, text(&env, "appeal notes"));

        let initial_event_count = env.events().all().len();
        resolve_appeal(&env, moderator, flag_id, true);
        let final_event_count = env.events().all().len();

        assert!(final_event_count > initial_event_count);
    }

    // ─── End-to-end flow tests ──────────────────────────────────────────────

    #[test]
    fn complete_moderation_workflow() {
        let (env, admin, moderator, user) = setup();

        // User flags content
        let flag_id = flag_content(
            &env,
            user.clone(),
            text(&env, "content_123"),
            FlagReason::Abusive,
            Some(text(&env, "Harassing language")),
        );
        assert_eq!(get_flag(&env, flag_id).unwrap().status, FlagStatus::Pending);

        // Moderator reviews and confirms
        review_flag(&env, moderator.clone(), flag_id, true);
        assert_eq!(
            get_flag(&env, flag_id).unwrap().status,
            FlagStatus::Confirmed
        );

        // Content owner appeals
        appeal_flag(&env, user.clone(), flag_id, text(&env, "False accusation"));
        assert_eq!(get_flag(&env, flag_id).unwrap().status, FlagStatus::Appealed);

        // Moderator grants appeal
        resolve_appeal(&env, moderator, flag_id, true);
        assert_eq!(
            get_flag(&env, flag_id).unwrap().status,
            FlagStatus::AppealGranted
        );
    }

    #[test]
    fn moderator_rejects_flag_workflow() {
        let (env, _, moderator, user) = setup();

        let flag_id = flag_content(
            &env,
            user,
            text(&env, "content_456"),
            FlagReason::Spam,
            Some(text(&env, "Legitimate promotional")),
        );

        review_flag(&env, moderator, flag_id, false);

        let flag = get_flag(&env, flag_id).unwrap();
        assert_eq!(flag.status, FlagStatus::Rejected);
    }
}
