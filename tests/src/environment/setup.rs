use std::{collections::HashMap, ops::Deref};

use near_sdk::Balance;
use near_units::parse_near;
use anyhow::Result;
use near_contract_standards::fungible_token;
// use futures::{stream::FuturesUnordered, TryStreamExt};
use near_sdk::{json_types::U128, serde_json::json, ONE_YOCTO};
use tokio::fs::read;
use workspaces::{network::Sandbox, testnet, Account, AccountId, Contract, Worker};

use crate::{
    WRAP_NEAR_TESTNET_ACCOUNT_ID, STREAMING_WASMS_DIR, EXTERNAL_TEST_WASMS_DIR, WRAP_NEAR_WASM,
    STREAMING_WASM, STREAMING_SUBACCOUNT_ID, FINANCE_WASM, FINANCE_SUBACCOUNT_ID,
    UTILITY_TOKEN_SUBACCOUNT_ID, UTILITY_TOKEN_WASM, UTILITY_TOKEN_DECIMALS,
};

use super::format_helpers::format_execution_result;

// Constant parameters used to setup the environment.
const DEFAULT_INITIAL_BALANCE: Balance = parse_near!("10.00 N");
const DEFAULT_STORAGE_DEPOSIT: Balance = parse_near!("0.0125 N");

// /// Creates account for the DAO orchestrating streaming contracts infrastructure.
// pub async fn create_dao_account(sandbox: Worker<Sandbox>) -> Result<Account> {
//     let dao = sandbox.dev_create_account().await?;
//     Ok(dao)
// }

/// Prepare w-near contract for the Sandbox. Either imports it from testnet or uses local wasm binary.
pub async fn prepare_wrap_near_contract(sandbox: Worker<Sandbox>) -> Result<Contract> {
    let id = WRAP_NEAR_TESTNET_ACCOUNT_ID.parse()?;
    let contract = match testnet().await {
        Ok(testnet) => {
            let contract = sandbox.import_contract(&id, &testnet).transact().await?;
            println!("wrap NEAR contract imported from testnet");
            contract
        }
        Err(e) => {
            println!("failed to connect to the testnet: {e}");
            println!("deploying local contract");
            let path = format!("{EXTERNAL_TEST_WASMS_DIR}/{WRAP_NEAR_WASM}");
            let wasm = read(path).await?;
            sandbox.dev_deploy(&wasm).await?
        }
    };

    let res = contract.call("new").transact().await?;
    println!(
        "\nwrapNEAR contract initialization outcome: {}\n",
        format_execution_result(&res)
    );

    Ok(contract)
}

/// Deploy streaming Roketo contract for selected DAO account.
pub async fn deploy_streaming_contract(dao: Account) -> Result<Contract> {
    let streaming_account = dao
        .create_subaccount(STREAMING_SUBACCOUNT_ID)
        .initial_balance(DEFAULT_INITIAL_BALANCE)
        .transact()
        .await?
        .into_result()?;

    let path = format!("{STREAMING_WASMS_DIR}/{STREAMING_WASM}");
    let wasm = read(path).await?;
    let streaming_contract = streaming_account
        .deploy(wasm.as_ref())
        .await?
        .into_result()?;

    println!(
        "deployed streaming contract under {}",
        streaming_account.id()
    );

    Ok(streaming_contract)
}

/// Deploy finance Roketo contract for selected DAO account.
pub async fn deploy_finance_contract(dao: Account) -> Result<Contract> {
    let finance_account = dao
        .create_subaccount(FINANCE_SUBACCOUNT_ID)
        .initial_balance(DEFAULT_INITIAL_BALANCE)
        .transact()
        .await?
        .into_result()?;

    let path = format!("{STREAMING_WASMS_DIR}/{FINANCE_WASM}");
    let wasm = read(path).await?;
    let finance_contract = finance_account.deploy(wasm.as_ref()).await?.into_result()?;

    println!("deployed finance contract under {}", finance_account.id());

    Ok(finance_contract)
}

/// Deploy utility Roketo token contract.
pub async fn deploy_utility_token_contract(dao: Account) -> Result<Contract> {
    let token_account = dao
        .create_subaccount(UTILITY_TOKEN_SUBACCOUNT_ID)
        .initial_balance(DEFAULT_INITIAL_BALANCE)
        .transact()
        .await?
        .into_result()?;

    let path = format!("{STREAMING_WASMS_DIR}/{UTILITY_TOKEN_WASM}");
    let wasm = read(path).await?;
    let token_contract = token_account.deploy(wasm.as_ref()).await?.into_result()?;

    let res = dao
        .call(token_contract.id(), "new")
        .args_json(json!({
            "owner_id": &dao.id(),
            "total_supply": "10000000000000000000",
            "metadata":  {
                "spec": "ft-1.0.0",
                "name": "Test Token",
                "symbol": "TST",
                "decimals": UTILITY_TOKEN_DECIMALS }
        }))
        .transact()
        .await?;
    assert!(res.is_success());

    println!(
        "\nRoketo utility token contract initialization outcome: {}\n",
        format_execution_result(&res)
    );

    Ok(token_contract)
}

/// Initialize Roketo contracts infrastructure.
/// On init calls they need to register each other.
pub async fn init_roketo_contracts(
    dao: Account,
    streaming: Contract,
    finance: Contract,
    fungible_tokens: HashMap<String, Contract>,
) -> Result<(Contract, Contract)> {
    let wrap_near = fungible_tokens.get(WRAP_NEAR_TESTNET_ACCOUNT_ID).unwrap();
    let roketo_ft = fungible_tokens.get(UTILITY_TOKEN_SUBACCOUNT_ID).unwrap();

    let res = dao
        .call(finance.id(), "new")
        .args_json((streaming.as_account().id(),))
        .transact()
        .await?;
    assert!(res.is_success());

    println!(
        "\nRoketo Finance contract initialization outcome: {}\n",
        format_execution_result(&res)
    );

    let res = dao
        .call(streaming.id(), "new")
        .args_json((
            dao.id(),
            finance.as_account().id(),
            roketo_ft.as_account().id(),
            UTILITY_TOKEN_DECIMALS,
        ))
        .transact()
        .await?;
    assert!(res.is_success());

    println!(
        "\nRoketo Streaming contract initialization outcome: {}\n",
        format_execution_result(&res)
    );

    Ok((streaming, finance))
}

/// Register fungible tokens on the streaming contract.
/// Return updated streaming contract.
pub async fn register_fts_on_streaming(
    dao: Account,
    streaming: Contract,
    fungible_tokens: HashMap<String, Contract>,
) -> Result<Contract> {
    let wrap_near = fungible_tokens.get(WRAP_NEAR_TESTNET_ACCOUNT_ID).unwrap();
    let roketo_ft = fungible_tokens.get(UTILITY_TOKEN_SUBACCOUNT_ID).unwrap();

    let res = dao
        .call(streaming.id(), "dao_update_token")
        .args_json(json!({
            "token": {
                "account_id": wrap_near.id(),
                "is_payment": true,
                "commission_on_transfer": "0",
                "commission_on_create": "10000",
                "commission_coef": { "val": 4, "pow": -3 },
                "collected_commission": "0",
                "storage_balance_needed": "0",
                "gas_for_ft_transfer": "200000000000000",
                "gas_for_storage_deposit": "200000000000000"
            }
        }))
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    println!(
        "\nEnlist wNEAR on the streaming outcome: {}\n",
        format_execution_result(&res)
    );

    let res = dao
        .call(streaming.id(), "dao_update_token")
        .args_json(json!({
            "token": {
                "account_id": roketo_ft.id(),
                "is_payment": false,
                "commission_on_transfer": "0",
                "commission_on_create": "10000",
                "commission_coef": { "val": 1, "pow": -2 },
                "collected_commission": "0",
                "storage_balance_needed": "0",
                "gas_for_ft_transfer": "200000000000000",
                "gas_for_storage_deposit": "200000000000000",
            }
        }))
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    println!(
        "\nEnlist Roketo Utility FT on the streaming outcome: {}\n",
        format_execution_result(&res)
    );

    Ok(streaming)
}

/// Register Roketo contract accounts on corresponding FTs
/// by transferring storage deposit amounts.
pub async fn add_storage_deposit(
    dao: Account,
    streaming: Contract,
    finance: Contract,
    mut fungible_tokens: HashMap<String, Contract>,
) -> Result<HashMap<String, Contract>> {
    let wrap_near = fungible_tokens
        .get(WRAP_NEAR_TESTNET_ACCOUNT_ID)
        .unwrap()
        .clone();
    let roketo_ft = fungible_tokens
        .get(UTILITY_TOKEN_SUBACCOUNT_ID)
        .unwrap()
        .clone();

    let res = dao
        .call(wrap_near.id(), "storage_deposit")
        .args_json(json!({
            "account_id": finance.id()
        }))
        .deposit(DEFAULT_STORAGE_DEPOSIT)
        .transact()
        .await?;
    assert!(res.is_success());

    println!(
        "\nStorage deposit for Finance on wNEAR outcome: {}\n",
        format_execution_result(&res)
    );

    let res = dao
        .call(wrap_near.id(), "storage_deposit")
        .args_json(json!({
            "account_id": streaming.id()
        }))
        .deposit(DEFAULT_STORAGE_DEPOSIT)
        .transact()
        .await?;
    assert!(res.is_success());

    println!(
        "\nStorage deposit for Streaming on wNEAR outcome: {}\n",
        format_execution_result(&res)
    );

    let res = dao
        .call(roketo_ft.id(), "storage_deposit")
        .args_json(json!({
            "account_id": finance.id()
        }))
        .deposit(DEFAULT_STORAGE_DEPOSIT)
        .transact()
        .await?;
    assert!(res.is_success());

    println!(
        "\nStorage deposit for Finance on Roketo FT outcome: {}\n",
        format_execution_result(&res)
    );

    let res = dao
        .call(roketo_ft.id(), "storage_deposit")
        .args_json(json!({
            "account_id": streaming.id()
        }))
        .deposit(DEFAULT_STORAGE_DEPOSIT)
        .transact()
        .await?;
    assert!(res.is_success());

    println!(
        "\nStorage deposit for Streaming on Roketo FT outcome: {}\n",
        format_execution_result(&res)
    );

    fungible_tokens.insert(WRAP_NEAR_TESTNET_ACCOUNT_ID.to_string(), wrap_near);
    fungible_tokens.insert(UTILITY_TOKEN_SUBACCOUNT_ID.to_string(), roketo_ft);
    Ok(fungible_tokens)
}
