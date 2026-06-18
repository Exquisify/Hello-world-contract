use soroban_sdk::{contractimpl, Env, String, TestEnv};

mod hello_world;

use hello_world::contract::{HelloContract, HelloContractClient};

const LEAP_YEAR: u32 = 2020;
const NON_LEAP_YEAR: u32 = 2019;
const NON_LEAP_CENTURY: u32 = 1900;
const LEAP_CENTURY: u32 = 2000;

#[test]
fn test_leap_year() {
    let env = TestEnv::new();
    let client = HelloContractClient::new(&env, &env.register_stellar_asset_contract(None));
    assert_eq!(client.leap_year(&env, LEAP_YEAR), true);
}

#[test]
fn test_non_leap_year() {
    let env = TestEnv::new();
    let client = HelloContractClient::new(&env, &env.register_stellar_asset_contract(None));
    assert_eq!(client.leap_year(&env, NON_LEAP_YEAR), false);
}

#[test]
fn test_non_leap_century() {
    let env = TestEnv::new();
    let client = HelloContractClient::new(&env, &env.register_stellar_asset_contract(None));
    assert_eq!(client.leap_year(&env, NON_LEAP_CENTURY), false);
}

#[test]
fn test_leap_century() {
    let env = TestEnv::new();
    let client = HelloContractClient::new(&env, &env.register_stellar_asset_contract(None));
    assert_eq!(client.leap_year(&env, LEAP_CENTURY), true);
}