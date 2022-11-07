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
const DEFAULT_INITIAL_BALANCE: Balance = parse_near!("20.00 N");
const DEFAULT_STORAGE_DEPOSIT: Balance = parse_near!("0.0125 N");

/// Extension for the testing environment consisting of the contracts
/// used for interoperation with `nft_benefits_vault`.
pub struct ExtVault {
    /// The account that issues NFT and pays benefits.
    pub issuer: Account,
    /// The account that owns NFT and receives benefits.
    pub nft_owner: Account,
    /// The Vault contract.
    pub vault: Contract,
    /// A simple NFT contract.
    pub nft: Contract,
}

/// NFT Issuer account registration.
pub async fn prepare_issuer_account(
    sandbox: Worker<Sandbox>,
    dao: Account,
    fungible_tokens: HashMap<&str, Contract>,
) -> Result<Account> {
    let issuer = dao
        .create_subaccount("issuer")
        .initial_balance(DEFAULT_INITIAL_BALANCE)
        .transact()
        .await?
        .into_result()?;

    // register_account(&issuer, tokens.iter().map(|t| t.id())).await?;
    let wrap_near = fungible_tokens
        .get(WRAP_NEAR_TESTNET_ACCOUNT_ID)
        .unwrap()
        .clone();

    let res = dao
        .call(wrap_near.id(), "storage_deposit")
        .args_json(json!({
            "account_id": issuer.id()
        }))
        .deposit(DEFAULT_STORAGE_DEPOSIT)
        .transact()
        .await?;
    assert!(res.is_success());

    // replenish_account_wrap_near(&issuer, tokens[0].id()).await?;
    let res = issuer
        .call(wrap_near.id(), "near_deposit")
        .deposit(parse_near!("10.00 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    println!(
        "deposit 10 NEAR of {} to {}: {}",
        wrap_near.id(),
        issuer.id(),
        format_execution_result(&res)
    );

    Ok(issuer)
}

/// NFT owner (and benefits receiver) account creation.
pub async fn prepare_nft_owner_account(
    sandbox: Worker<Sandbox>,
    dao: Account,
    fungible_tokens: HashMap<&str, Contract>,
) -> Result<Account> {
    let nft_owner = dao
        .create_subaccount("nft_owner")
        .initial_balance(parse_near!("0.1 N"))
        .transact()
        .await?
        .into_result()?;

    // register_account
    let wrap_near = fungible_tokens
        .get(WRAP_NEAR_TESTNET_ACCOUNT_ID)
        .unwrap()
        .clone();

    let res = dao
        .call(wrap_near.id(), "storage_deposit")
        .args_json(json!({
            "account_id": nft_owner.id()
        }))
        .deposit(DEFAULT_STORAGE_DEPOSIT)
        .transact()
        .await?;
    assert!(res.is_success());

    println!(
        "\nStorage deposit for NFT_Owner on wNEAR outcome: {}\n",
        format_execution_result(&res)
    );

    Ok(nft_owner)
}

/// Import external vault contract as pre-built artifact
/// and initialize it with wNEAR.
pub async fn prepare_external_vault_contract(
    sandbox: Worker<Sandbox>,
    dao: Account,
    fungible_tokens: HashMap<&str, Contract>,
) -> Result<Contract> {
    let path = format!("{EXTERNAL_TEST_WASMS_DIR}/nft_benefits_vault.wasm");
    println!("read Vault contract WASM code from: {path}");

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

    println!(
        "\nStorage deposit for Vault on wNEAR outcome: {}\n",
        format_execution_result(&res)
    );

    Ok(contract)
}

/// Import external NFT contract as pre-built artifact.
pub async fn prepare_external_nft_contract(
    sandbox: Worker<Sandbox>,
    dao: Account,
    fungible_tokens: HashMap<&str, Contract>,
) -> Result<Contract> {
    let path = format!("{EXTERNAL_TEST_WASMS_DIR}/non_fungible_token.wasm");
    println!("read NFT contract WASM code from: {path}");

    let wasm = read(path).await?;
    println!("NFT WASM code imported");

    let contract = sandbox.dev_deploy(&wasm).await?;
    println!("NFT WASM code deployed");

    let args = json!({
        "owner_id": contract.id(),
    });

    let res = contract
        .call("new_default_meta")
        .args_json(args)
        .transact()
        .await?;
    println!(
        "NFT contract initialization: {}",
        format_execution_result(&res)
    );

    Ok(contract)
}
