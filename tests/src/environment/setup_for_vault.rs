use std::collections::HashMap;

use near_sdk::Balance;
use near_units::parse_near;
use anyhow::Result;
use near_sdk::{serde_json::json, ONE_YOCTO};
use tokio::fs::read;
use workspaces::{network::Sandbox, testnet, Account, Contract, Worker};

use crate::{
    WRAP_NEAR_TESTNET_ACCOUNT_ID, STREAMING_WASMS_DIR, EXTERNAL_TEST_WASMS_DIR, WRAP_NEAR_WASM,
    STREAMING_WASM, STREAMING_SUBACCOUNT_ID, FINANCE_WASM, FINANCE_SUBACCOUNT_ID,
    UTILITY_TOKEN_SUBACCOUNT_ID, UTILITY_TOKEN_WASM, UTILITY_TOKEN_DECIMALS,
};

use super::format_helpers::format_execution_result;

// Constant parameters used to setup the environment.
const DEFAULT_INITIAL_BALANCE: Balance = parse_near!("10.00 N");
const DEFAULT_STORAGE_DEPOSIT: Balance = parse_near!("0.0125 N");

/// Extension for the testing environment consisting of the contracts
/// used for interoperation with `nft_benefits_vault`.
pub struct ExtVault {
    /// The Vault contract
    pub vault: Contract,
}

/// Import external vault contract as pre-built artifact
/// and initialize it with wNEAR.
pub async fn prepare_external_vault_contract(
    sandbox: Worker<Sandbox>,
    dao: Account,
    mut fungible_tokens: HashMap<String, Contract>,
) -> Result<Contract> {
    let path = format!("{EXTERNAL_TEST_WASMS_DIR}/nft_benefits_vault.wasm");
    println!("read WASM contract code from: {path}");

    let wasm = read(path).await?;
    println!("vault WASM code imported");

    let contract = sandbox.dev_deploy(&wasm).await?;
    println!("vault WASM code deployed");

    // register_account(contract.as_account(), tokens.iter().map(|t| t.id())).await?;
    let wrap_near = fungible_tokens
        .get(WRAP_NEAR_TESTNET_ACCOUNT_ID)
        .unwrap()
        .clone();

    let res = dao
        .call(wrap_near.id(), "storage_deposit")
        .args_json(json!({
            "account_id": contract.id()
        }))
        .deposit(DEFAULT_STORAGE_DEPOSIT)
        .transact()
        .await?;
    assert!(res.is_success());

    println!(
        "\nStorage deposit for Vault on wNEAR outcome: {}\n",
        format_execution_result(&res)
    );

    Ok(contract)
}
