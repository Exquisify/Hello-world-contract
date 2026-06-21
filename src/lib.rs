#![no_std]

#[cfg(test)]
extern crate std;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env, String, Vec,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdeaData {
    pub id: i128,
    pub author: Address,
    pub title: String,
    pub body: String,
    pub category: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub deleted: bool,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    NextIdeaId,
    Idea(i128),
    AuthorIdeas(Address),
    CategoryIdeas(String),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum IdeaError {
    NotFound = 1,
    Unauthorized = 2,
    Deleted = 3,
}

#[contract]
pub struct IdeaContract;

#[contractimpl]
impl IdeaContract {
    pub fn create_idea(
        env: Env,
        author: Address,
        title: String,
        body: String,
        category: String,
    ) -> i128 {
        author.require_auth();

        let idea_id = next_idea_id(&env);
        let now = env.ledger().timestamp();
        let idea = IdeaData {
            id: idea_id,
            author: author.clone(),
            title,
            body,
            category: category.clone(),
            created_at: now,
            updated_at: now,
            deleted: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Idea(idea_id), &idea);
        append_id(&env, DataKey::AuthorIdeas(author.clone()), idea_id);
        append_id(&env, DataKey::CategoryIdeas(category), idea_id);

        env.events()
            .publish((symbol_short!("Created"), author), idea_id);

        idea_id
    }

    pub fn read_idea(env: Env, idea_id: i128) -> IdeaData {
        get_idea(&env, idea_id)
    }

    pub fn update_idea(env: Env, idea_id: i128, caller: Address, new_body: String) {
        caller.require_auth();

        let mut idea = get_idea(&env, idea_id);
        if idea.deleted {
            panic_with_error!(&env, IdeaError::Deleted);
        }
        if idea.author != caller {
            panic_with_error!(&env, IdeaError::Unauthorized);
        }

        idea.body = new_body;
        idea.updated_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Idea(idea_id), &idea);

        env.events()
            .publish((symbol_short!("Updated"), caller), idea_id);
    }

    pub fn delete_idea(env: Env, idea_id: i128, caller: Address) {
        caller.require_auth();

        let mut idea = get_idea(&env, idea_id);
        if idea.deleted {
            panic_with_error!(&env, IdeaError::Deleted);
        }
        if idea.author != caller {
            panic_with_error!(&env, IdeaError::Unauthorized);
        }

        idea.deleted = true;
        idea.updated_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::Idea(idea_id), &idea);

        env.events()
            .publish((symbol_short!("Deleted"), caller), idea_id);
    }

    pub fn list_ideas_by_author(env: Env, author: Address) -> Vec<i128> {
        env.storage()
            .persistent()
            .get(&DataKey::AuthorIdeas(author))
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn list_ideas_by_category(env: Env, category: String) -> Vec<i128> {
        env.storage()
            .persistent()
            .get(&DataKey::CategoryIdeas(category))
            .unwrap_or_else(|| Vec::new(&env))
    }
}

fn next_idea_id(env: &Env) -> i128 {
    let current = env
        .storage()
        .persistent()
        .get(&DataKey::NextIdeaId)
        .unwrap_or(0_i128);
    let next = current + 1;
    env.storage().persistent().set(&DataKey::NextIdeaId, &next);
    next
}

fn append_id(env: &Env, key: DataKey, idea_id: i128) {
    let mut ids = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    ids.push_back(idea_id);
    env.storage().persistent().set(&key, &ids);
}

fn get_idea(env: &Env, idea_id: i128) -> IdeaData {
    env.storage()
        .persistent()
        .get(&DataKey::Idea(idea_id))
        .unwrap_or_else(|| panic_with_error!(env, IdeaError::NotFound))
}

#[cfg(test)]
mod tests;
