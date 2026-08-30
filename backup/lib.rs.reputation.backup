#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

const INITIAL_REPUTATION: u64 = 100;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Reputation(Address),
    Admin,
    VoteCount(Symbol),
    WeightedVotes(Symbol),
}

#[contract]
pub struct ReputationContract;

#[contractimpl]
impl ReputationContract {
    /// Initialize admin
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// Query user reputation (defaults to INITIAL_REPUTATION if unassigned)
    pub fn get_reputation(env: Env, user: Address) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::Reputation(user))
            .unwrap_or(INITIAL_REPUTATION)
    }

    /// Reward a user for high quality contributions or accurate predictions
    pub fn add_reputation(env: Env, _admin: Address, user: Address, amount: u64) {
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        stored_admin.require_auth();

        let current = Self::get_reputation(env.clone(), user.clone());
        let new_rep = current.saturating_add(amount);
        env.storage().persistent().set(&DataKey::Reputation(user), &new_rep);
    }

    /// Penalize a user for low quality content or incorrect predictions
    pub fn deduct_reputation(env: Env, _admin: Address, user: Address, amount: u64) {
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        stored_admin.require_auth();

        let current = Self::get_reputation(env.clone(), user.clone());
        let new_rep = current.saturating_sub(amount);
        env.storage().persistent().set(&DataKey::Reputation(user), &new_rep);
    }

    /// Cast a vote weighted by the user's current reputation score
    pub fn cast_weighted_vote(env: Env, voter: Address, proposal_id: Symbol) -> u64 {
        voter.require_auth();
        let reputation = Self::get_reputation(env.clone(), voter);

        let current_votes: u64 = env.storage().persistent().get(&DataKey::WeightedVotes(proposal_id.clone())).unwrap_or(0);
        let updated_votes = current_votes.saturating_add(reputation);

        env.storage().persistent().set(&DataKey::WeightedVotes(proposal_id.clone()), &updated_votes);

        let count: u64 = env.storage().persistent().get(&DataKey::VoteCount(proposal_id.clone())).unwrap_or(0);
        env.storage().persistent().set(&DataKey::VoteCount(proposal_id), &(count + 1));

        updated_votes
    }

    /// Query total weighted votes for a proposal
    pub fn get_proposal_votes(env: Env, proposal_id: Symbol) -> u64 {
        env.storage().persistent().get(&DataKey::WeightedVotes(proposal_id)).unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_reputation_flow_and_weighted_voting() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ReputationContract);
        let client = ReputationContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        env.mock_all_auths();

        client.initialize(&admin);

        // Verify initial reputation default
        assert_eq!(client.get_reputation(&user1), 100);

        // Increase reputation for positive contribution
        client.add_reputation(&admin, &user1, &50);
        assert_eq!(client.get_reputation(&user1), 150);

        // Deduct reputation for poor contribution
        client.deduct_reputation(&admin, &user2, &30);
        assert_eq!(client.get_reputation(&user2), 70);

        // Cast reputation-weighted votes
        let proposal = symbol_short!("prop1");
        client.cast_weighted_vote(&user1, &proposal);
        client.cast_weighted_vote(&user2, &proposal);

        // Total weighted votes = 150 + 70 = 220
        assert_eq!(client.get_proposal_votes(&proposal), 220);
    }
}
