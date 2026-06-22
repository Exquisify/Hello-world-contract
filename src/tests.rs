use super::{
    IdeaContract, IdeaContractClient, IdeaError, VoteCount, VoteDirection, VoteRecord,
};
use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, Events},
    Address, Env, String,
};

#[contract]
pub struct MockReputationContract;

#[contractimpl]
impl MockReputationContract {
    pub fn reputation_score(_env: Env, _user: Address) -> i32 {
        // 200 / 100 = 2 vote weight
        200
    }
}

fn setup() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(IdeaContract, ());
    let reputation_contract_id = env.register(MockReputationContract, ());
    let author = Address::generate(&env);
    let other = Address::generate(&env);

    let client = IdeaContractClient::new(&env, &contract_id);
    client.set_reputation_contract(&reputation_contract_id);

    (env, contract_id, reputation_contract_id, author, other)
}

fn setup_without_reputation() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(IdeaContract, ());
    let author = Address::generate(&env);
    let other = Address::generate(&env);

    (env, contract_id, author, other)
}

fn text(env: &Env, value: &str) -> String {
    String::from_str(env, value)
}

#[test]
fn create_and_read_idea_stores_metadata() {
    let (env, contract_id, _, author, _) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "BTC breakout plan"),
        &text(&env, "Wait for volume confirmation before entry."),
        &text(&env, "technical-analysis"),
    );

    let idea = client.read_idea(&idea_id);

    assert_eq!(idea_id, 1);
    assert_eq!(idea.id, idea_id);
    assert_eq!(idea.author, author);
    assert_eq!(idea.title, text(&env, "BTC breakout plan"));
    assert_eq!(idea.body, text(&env, "Wait for volume confirmation before entry."));
    assert_eq!(idea.category, text(&env, "technical-analysis"));
    assert_eq!(idea.created_at, env.ledger().timestamp());
    assert_eq!(idea.updated_at, env.ledger().timestamp());
    assert_eq!(idea.deleted, false);
}

#[test]
fn list_ideas_by_author_and_category_returns_matching_ids() {
    let (env, contract_id, _, author, other) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let first = client.create_idea(
        &author,
        &text(&env, "ETH range"),
        &text(&env, "Fade extremes until invalidation."),
        &text(&env, "strategy"),
    );
    let second = client.create_idea(
        &author,
        &text(&env, "SOL momentum"),
        &text(&env, "Track funding and open interest."),
        &text(&env, "strategy"),
    );
    let third = client.create_idea(
        &other,
        &text(&env, "Macro risk"),
        &text(&env, "Reduce leverage around major data releases."),
        &text(&env, "risk"),
    );

    let author_ids = client.list_ideas_by_author(&author);
    assert_eq!(author_ids.len(), 2);
    assert_eq!(author_ids.get(0).unwrap(), first);
    assert_eq!(author_ids.get(1).unwrap(), second);

    let strategy_ids = client.list_ideas_by_category(&text(&env, "strategy"));
    assert_eq!(strategy_ids.len(), 2);
    assert_eq!(strategy_ids.get(0).unwrap(), first);
    assert_eq!(strategy_ids.get(1).unwrap(), second);

    let other_ids = client.list_ideas_by_author(&other);
    assert_eq!(other_ids.len(), 1);
    assert_eq!(other_ids.get(0).unwrap(), third);
}

#[test]
fn author_can_update_body_without_changing_metadata() {
    let (env, contract_id, _, author, _) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "BTC thesis"),
        &text(&env, "Initial plan."),
        &text(&env, "insight"),
    );

    client.update_idea(&idea_id, &author, &text(&env, "Updated plan with tighter risk."));

    let idea = client.read_idea(&idea_id);
    assert_eq!(idea.title, text(&env, "BTC thesis"));
    assert_eq!(idea.body, text(&env, "Updated plan with tighter risk."));
    assert_eq!(idea.category, text(&env, "insight"));
    assert_eq!(idea.author, author);
    assert_eq!(idea.deleted, false);
}

#[test]
fn delete_idea_soft_deletes_record() {
    let (env, contract_id, _, author, _) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "Exit plan"),
        &text(&env, "Scale out after volatility expansion."),
        &text(&env, "strategy"),
    );

    client.delete_idea(&idea_id, &author);

    let idea = client.read_idea(&idea_id);
    assert_eq!(idea.deleted, true);
    assert_eq!(idea.body, text(&env, "Scale out after volatility expansion."));

    let author_ids = client.list_ideas_by_author(&author);
    assert_eq!(author_ids.len(), 1);
    assert_eq!(author_ids.get(0).unwrap(), idea_id);
}

#[test]
fn update_by_non_author_returns_unauthorized_error() {
    let (env, contract_id, _, author, other) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "Only author can edit"),
        &text(&env, "Original body."),
        &text(&env, "insight"),
    );

    assert_eq!(
        client.try_update_idea(&idea_id, &other, &text(&env, "Unauthorized body.")),
        Err(Ok(IdeaError::Unauthorized))
    );
}

#[test]
fn delete_by_non_author_returns_unauthorized_error() {
    let (env, contract_id, _, author, other) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "Only author can delete"),
        &text(&env, "Original body."),
        &text(&env, "insight"),
    );

    assert_eq!(
        client.try_delete_idea(&idea_id, &other),
        Err(Ok(IdeaError::Unauthorized))
    );
}

#[test]
fn read_missing_idea_returns_not_found_error() {
    let (env, contract_id, _, _, _) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    assert_eq!(client.try_read_idea(&404), Err(Ok(IdeaError::NotFound)));
}

#[test]
fn update_deleted_idea_returns_deleted_error() {
    let (env, contract_id, _, author, _) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "Deleted idea"),
        &text(&env, "Original body."),
        &text(&env, "insight"),
    );
    client.delete_idea(&idea_id, &author);

    assert_eq!(
        client.try_update_idea(&idea_id, &author, &text(&env, "Cannot edit deleted.")),
        Err(Ok(IdeaError::Deleted))
    );
}

#[test]
fn create_update_delete_emit_events() {
    let (env, contract_id, _, author, _) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "Event coverage"),
        &text(&env, "Initial body."),
        &text(&env, "testing"),
    );
    client.update_idea(&idea_id, &author, &text(&env, "Updated body."));
    client.delete_idea(&idea_id, &author);

    assert_eq!(env.events().all().len(), 3);
}

#[test]
fn exposes_contract_error_codes() {
    assert_eq!(IdeaError::NotFound as u32, 1);
    assert_eq!(IdeaError::Unauthorized as u32, 2);
    assert_eq!(IdeaError::Deleted as u32, 3);
    assert_eq!(IdeaError::VoteAlreadyExists as u32, 4);
    assert_eq!(IdeaError::VoteNotFound as u32, 5);
    assert_eq!(IdeaError::ReputationContractNotConfigured as u32, 6);
    assert_eq!(IdeaError::InvalidVoteWeight as u32, 7);
}

#[test]
fn vote_records_weighted_upvote_and_updates_counts() {
    let (env, contract_id, _, author, other) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "Weighted vote idea"),
        &text(&env, "Voting should be weighted by reputation."),
        &text(&env, "governance"),
    );

    client.vote(&idea_id, &other, &VoteDirection::Up);

    let vote = client.get_vote(&idea_id, &other).unwrap();
    assert_eq!(
        vote,
        VoteRecord {
            direction: VoteDirection::Up,
            weight: 2
        }
    );

    let counts = client.get_vote_count(&idea_id);
    assert_eq!(
        counts,
        VoteCount {
            upvotes: 2,
            downvotes: 0,
            net_score: 2
        }
    );
}

#[test]
fn vote_records_weighted_downvote_and_updates_counts() {
    let (env, contract_id, _, author, other) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "Downvote idea"),
        &text(&env, "This one tests weighted downvotes."),
        &text(&env, "governance"),
    );

    client.vote(&idea_id, &other, &VoteDirection::Down);

    let vote = client.get_vote(&idea_id, &other).unwrap();
    assert_eq!(
        vote,
        VoteRecord {
            direction: VoteDirection::Down,
            weight: 2
        }
    );

    let counts = client.get_vote_count(&idea_id);
    assert_eq!(
        counts,
        VoteCount {
            upvotes: 0,
            downvotes: 2,
            net_score: -2
        }
    );
}

#[test]
fn change_vote_switches_direction_and_recomputes_counts() {
    let (env, contract_id, _, author, other) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "Vote switching"),
        &text(&env, "Change vote from up to down."),
        &text(&env, "governance"),
    );

    client.vote(&idea_id, &other, &VoteDirection::Up);
    client.change_vote(&idea_id, &other, &VoteDirection::Down);

    let vote = client.get_vote(&idea_id, &other).unwrap();
    assert_eq!(
        vote,
        VoteRecord {
            direction: VoteDirection::Down,
            weight: 2
        }
    );

    let counts = client.get_vote_count(&idea_id);
    assert_eq!(
        counts,
        VoteCount {
            upvotes: 0,
            downvotes: 2,
            net_score: -2
        }
    );
}

#[test]
fn remove_vote_retracts_vote_and_resets_counts() {
    let (env, contract_id, _, author, other) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "Retract vote"),
        &text(&env, "Vote can be removed."),
        &text(&env, "governance"),
    );

    client.vote(&idea_id, &other, &VoteDirection::Up);
    client.remove_vote(&idea_id, &other);

    let vote = client.get_vote(&idea_id, &other);
    assert_eq!(vote, None);

    let counts = client.get_vote_count(&idea_id);
    assert_eq!(
        counts,
        VoteCount {
            upvotes: 0,
            downvotes: 0,
            net_score: 0
        }
    );
}

#[test]
fn double_voting_is_prevented() {
    let (env, contract_id, _, author, other) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "Prevent duplicate votes"),
        &text(&env, "Same voter should not be able to vote twice."),
        &text(&env, "governance"),
    );

    client.vote(&idea_id, &other, &VoteDirection::Up);

    assert_eq!(
        client.try_vote(&idea_id, &other, &VoteDirection::Down),
        Err(Ok(IdeaError::VoteAlreadyExists))
    );
}

#[test]
fn change_vote_without_existing_vote_returns_error() {
    let (env, contract_id, _, author, other) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "Missing vote change"),
        &text(&env, "Changing a missing vote should fail."),
        &text(&env, "governance"),
    );

    assert_eq!(
        client.try_change_vote(&idea_id, &other, &VoteDirection::Down),
        Err(Ok(IdeaError::VoteNotFound))
    );
}

#[test]
fn remove_vote_without_existing_vote_returns_error() {
    let (env, contract_id, _, author, other) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "Missing vote removal"),
        &text(&env, "Removing a missing vote should fail."),
        &text(&env, "governance"),
    );

    assert_eq!(
        client.try_remove_vote(&idea_id, &other),
        Err(Ok(IdeaError::VoteNotFound))
    );
}

#[test]
fn vote_without_reputation_contract_returns_error() {
    let (env, contract_id, author, other) = setup_without_reputation();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "No reputation contract"),
        &text(&env, "Voting should fail without reputation contract."),
        &text(&env, "governance"),
    );

    assert_eq!(
        client.try_vote(&idea_id, &other, &VoteDirection::Up),
        Err(Ok(IdeaError::ReputationContractNotConfigured))
    );
}

#[test]
fn vote_change_remove_emit_events() {
    let (env, contract_id, _, author, other) = setup();
    let client = IdeaContractClient::new(&env, &contract_id);

    let idea_id = client.create_idea(
        &author,
        &text(&env, "Voting events"),
        &text(&env, "Ensure vote events are emitted."),
        &text(&env, "testing"),
    );

    // create_idea emits 1 event
    client.vote(&idea_id, &other, &VoteDirection::Up);
    client.change_vote(&idea_id, &other, &VoteDirection::Down);
    client.remove_vote(&idea_id, &other);

    // 1 create + 3 voting events
    assert_eq!(env.events().all().len(), 4);
}
