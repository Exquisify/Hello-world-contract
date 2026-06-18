use soroban_sdk::{contract, contractimpl, Env, String};

#[contract]
pub struct HelloContract;

#[contractimpl]
impl HelloContract {
    pub fn hello(env: Env, name: String) -> String {
        if name.is_empty() {
            String::from_str(&env, "Hello, world!")
        } else {
            String::from_str(&env, "Hello, ") + &name + String::from_str(&env, "!")
        }
    }

    pub fn leap_year(env: Env, year: u32) -> bool {
        let div4 = year % 4 == 0;
        let div100 = year % 100 == 0;
        let div400 = year % 400 == 0;
        (div4 && !div100) || div400
    }
}