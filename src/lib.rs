#![no_std]

pub mod wrapped_token;
pub use wrapped_token::{WrapperContract, WrapperContractClient, WTokenKey};

pub mod moderation;
pub use moderation::{
    initialize_moderation, add_moderator, remove_moderator, flag_content, review_flag,
    appeal_flag, resolve_appeal, get_flag, get_flag_count, is_moderator, ContentFlag,
    FlagStatus, FlagReason, ModerationError, ModerationKey,
};

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Token,
    Admin,
    ClaimLimit,
    Cooldown,
    LastClaim(Address),
}

#[contract]
pub struct FaucetContract;

#[contractimpl]
impl FaucetContract {
    /// Initialize the faucet contract.
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `admin` - The address of the admin
    /// * `token` - The address of the token to dispense
    /// * `claim_limit` - The maximum amount of tokens allowed per claim
    /// * `cooldown` - The cooldown period in seconds between claims
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        claim_limit: i128,
        cooldown: u64,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::ClaimLimit, &claim_limit);
        env.storage().instance().set(&DataKey::Cooldown, &cooldown);
    }

    /// Claim tokens from the faucet.
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `user` - The address claiming the tokens
    /// * `amount` - The amount of tokens to claim
    pub fn claim(env: Env, user: Address, amount: i128) {
        user.require_auth();
        
        let claim_limit: i128 = env.storage().instance().get(&DataKey::ClaimLimit).unwrap();
        if amount > claim_limit {
            panic!("claim amount exceeds limit");
        }

        let cooldown: u64 = env.storage().instance().get(&DataKey::Cooldown).unwrap();
        let current_time = env.ledger().timestamp();

        let last_claim = env
            .storage()
            .persistent()
            .get::<_, u64>(&DataKey::LastClaim(user.clone()))
            .unwrap_or(0);

        if current_time < last_claim + cooldown {
            panic!("cooldown period has not expired");
        }

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_addr);

        token_client.transfer(&env.current_contract_address(), &user, &amount);

        env.storage()
            .persistent()
            .set(&DataKey::LastClaim(user), &current_time);
    }

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::token::StellarAssetClient;

    #[test]
    fn test_claim() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        
        let contract_id = env.register(FaucetContract, ());
        let client = FaucetContractClient::new(&env, &contract_id);
        
        let token_admin = Address::generate(&env);
        let token_id = env.register_stellar_asset_contract(token_admin.clone());
        let token_client = token::Client::new(&env, &token_id);
        let token_admin_client = StellarAssetClient::new(&env, &token_id);
        
        let claim_limit = 100;
        let cooldown = 60;
        
        client.initialize(&admin, &token_id, &claim_limit, &cooldown);
        
        // mint some tokens to the contract
        token_admin_client.mint(&contract_id, &1000);
        
        // Initial claim
        env.ledger().set_timestamp(100);
        client.claim(&user, &50);
        assert_eq!(token_client.balance(&user), 50);
    }

    #[test]
    #[should_panic(expected = "claim amount exceeds limit")]
    fn test_limit_enforcement() {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        
        let contract_id = env.register(FaucetContract, ());
        let client = FaucetContractClient::new(&env, &contract_id);
        
        let token_admin = Address::generate(&env);
        let token_id = env.register_stellar_asset_contract(token_admin.clone());
        let token_admin_client = StellarAssetClient::new(&env, &token_id);
        
        client.initialize(&admin, &token_id, &100, &60);
        token_admin_client.mint(&contract_id, &1000);
        
        client.claim(&user, &150);
    }

    #[test]
    #[should_panic(expected = "cooldown period has not expired")]
    fn test_cooldown_check() {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        
        let contract_id = env.register(FaucetContract, ());
        let client = FaucetContractClient::new(&env, &contract_id);
        
        let token_admin = Address::generate(&env);
        let token_id = env.register_stellar_asset_contract(token_admin.clone());
        let token_admin_client = StellarAssetClient::new(&env, &token_id);
        
        client.initialize(&admin, &token_id, &100, &60);
        token_admin_client.mint(&contract_id, &1000);
        
        env.ledger().set_timestamp(100);
        client.claim(&user, &50);
        
        // Attempt to claim again immediately
        client.claim(&user, &50);
    }
}
