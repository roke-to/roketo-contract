use std::collections::HashMap;

use common::m_e_n;
use near_contract_standards::fungible_token;
use near_sdk::Balance;
use near_units::parse_near;
use anyhow::Result;
use near_sdk::{serde_json::json, ONE_YOCTO, ONE_NEAR};
use tokio::fs::read;
use workspaces::{network::Sandbox, testnet, Account, Contract, Worker};

use crate::{
    WRAP_NEAR_TESTNET_ACCOUNT_ID, STREAMING_WASMS_DIR, EXTERNAL_TEST_WASMS_DIR, WRAP_NEAR_WASM,
    STREAMING_WASM, STREAMING_SUBACCOUNT_ID, FINANCE_WASM, FINANCE_SUBACCOUNT_ID,
    UTILITY_TOKEN_SUBACCOUNT_ID, UTILITY_TOKEN_WASM, UTILITY_TOKEN_DECIMALS,
    environment::methods::{ft_storage_deposit, mint_ft},
};

use super::format_helpers::format_execution_result;

// Constant parameters used to setup the environment.
const DEFAULT_INITIAL_BALANCE: Balance = parse_near!("10.00 N");
const DEFAULT_STORAGE_DEPOSIT: Balance = parse_near!("0.0125 N");
const USER_INITIAL_BALANCE: Balance = 10000 * ONE_NEAR;

/// Extension for the testing environment for porting `near_sdk_sim`.
#[derive(Clone)]
// pub struct ExtIntegration<'a> {
//     /// A collection of test users to be created
//     /// under environment's dao master account.
//     pub users: HashMap<&'a str, Account>,
// }
pub struct ExtIntegration {
    /// A collection of test users to be created
    /// under environment's dao master account.
    pub users: HashMap<String, Account>,
}

// pub async fn prepare_users(sandbox: Worker<Sandbox>) -> Result<HashMap<&'static str, Account>> {
pub async fn prepare_users(sandbox: Worker<Sandbox>) -> Result<HashMap<String, Account>> {
    let mut users: HashMap<String, Account> = HashMap::new();
    let root = sandbox.root_account()?;

    // for username in ["alice", "bob", "charlie", "dude", "eve"] {
    for username in ["alice", "bob"] {
        let user = root
            .create_subaccount(username)
            .initial_balance(USER_INITIAL_BALANCE)
            .transact()
            .await?
            .into_result()?;
        users.insert(username.to_string(), user);
        println!("\nprepared test user {}\n", username);
    }
    Ok(users)
}

pub async fn mint_tokens(
    sandbox: &Worker<Sandbox>,
    fungible_tokens: &HashMap<&str, Contract>,
    users: &HashMap<String, Account>,
    amount: u32,
) -> Result<()> {
    let wrap_near = fungible_tokens
        .get(WRAP_NEAR_TESTNET_ACCOUNT_ID)
        .unwrap()
        .clone();

    //@TODO: make the following property list constant for the test suite.
    //@TODO: parallelize inner calls over this list.
    // for username in ["alice", "bob", "charlie", "dude", "eve"] {
    for username in ["alice", "bob"] {
        let user = users.get(&username.to_string()).unwrap();
        let res = ft_storage_deposit(user, wrap_near.id(), user.id()).await?;
        mint_ft(sandbox, wrap_near.id(), user, m_e_n(amount, 24)).await?;
        println!("\nminted {amount} of wNEAR for {}\n", username);
    }
    Ok(())
}
