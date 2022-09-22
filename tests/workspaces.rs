use serde_json::json;
use near_sdk::ONE_YOCTO;
use near_units::parse_near;
use workspaces::prelude::*;
use workspaces::network::Sandbox;
use workspaces::{Account, AccountId, Contract, Worker};
pub use near_sdk::json_types::U128;
use streaming::{Stream, StreamStatus, AccountView};
use tokio::time::{sleep, Duration};
// use workspaces::DevNetwork;

use streaming::Dao;

async fn pull_contract(
    owner: &Account,
    contract_account: &str,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<Contract> {
    let testnet = workspaces::testnet_archival().await?;
    let contract_id: AccountId = contract_account.parse::<AccountId>().unwrap();
    let contract = worker
        .import_contract(&contract_id, &testnet)
        .initial_balance(parse_near!("1000 N"))
        // .block_height(BLOCK_HEIGHT)
        .transact()
        .await?;

    owner
        .call(worker, contract.id(), "new")
        // .deposit(ONE_YOCTO)
        .transact()
        .await?;

    Ok(contract)
}

async fn init(
    // TODO:
    // worker: &Worker<impl DevNetwork>,
    worker: &Worker<workspaces::network::Sandbox>,
    dao_account: &Account,
) -> anyhow::Result<(Contract, Contract, Contract, Contract)> {
    // let master_account = worker.dev_create_account().await?;

    // deployment of contracts
    let streaming_contract = worker
        .dev_deploy(include_bytes!("../res/streaming.wasm").as_ref())
        .await?;

    let finance_contract = worker
        .dev_deploy(include_bytes!("../res/finance.wasm").as_ref())
        .await?;

    let token_contract = worker
        .dev_deploy(include_bytes!("../res/roke_token.wasm").as_ref())
        .await?;

    // spoon & deploy the `wrap.testnet` contract
    let wrap_testnet = pull_contract(dao_account, "wrap.testnet", worker).await;
    println!("Wrap spooning result: {:?}", wrap_testnet);
    assert!(wrap_testnet.is_ok());
    let wrap_contract = wrap_testnet?;

    // init finance and streaming contracts
    let res = dao_account
        .call(worker, finance_contract.id(), "new")
        .args_json((streaming_contract.as_account().id(),))?
        .transact()
        .await?;
    assert!(res.is_success());

    let res = dao_account
        .call(worker, streaming_contract.id(), "new")
        .args_json((
            dao_account.id(),
            streaming_contract.as_account().id(),
            token_contract.as_account().id(),
            18,
        ))?
        .transact()
        .await?;
    assert!(res.is_success());

    // register roketo FT
    let res = dao_account
        .call(worker, token_contract.id(), "new")
        .args_json(json!({
            "owner_id": &dao_account.id(),
            "total_supply": "10000000000000000000",
            "metadata":  {
                "spec": "ft-1.0.0",
                "name": "Test Token",
                "symbol": "TST",
                "decimals": 18 }
        }))?
        .transact()
        .await?;
    assert!(res.is_success());

    // adding ft to streaming
    // add wrap token
    let res = dao_account
        .call(worker, streaming_contract.id(), "dao_update_token")
        .args_json(json!({
            "token": {
                "account_id": wrap_contract.id(),
                "is_payment": true,
                "commission_on_transfer": "0",
                "commission_on_create": "10000",
                "commission_coef": { "val": 4, "pow": -3 },
                "collected_commission": "0",
                "storage_balance_needed": "0",
                "gas_for_ft_transfer": "200000000000000",
                "gas_for_storage_deposit": "200000000000000"
            }
        }))?
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    // add util token
    let res = dao_account
        .call(worker, streaming_contract.id(), "dao_update_token")
        .args_json(json!({
            "token": {
                "account_id": token_contract.id(),
                "is_payment": false,
                "commission_on_transfer": "0",
                "commission_on_create": "10000",
                "commission_coef": { "val": 1, "pow": -2 },
                "collected_commission": "0",
                "storage_balance_needed": "0",
                "gas_for_ft_transfer": "200000000000000",
                "gas_for_storage_deposit": "200000000000000",
            }
        }))?
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    // register contract accounts
    // storage deposit in each FT for finance and streaming accounts
    let res = dao_account
        .call(worker, wrap_contract.id(), "storage_deposit")
        .args_json(json!({
            "account_id": finance_contract.id()
        }))?
        .deposit(parse_near!("0.0125 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    let res = dao_account
        .call(worker, wrap_contract.id(), "storage_deposit")
        .args_json(json!({
            "account_id": streaming_contract.id()
        }))?
        .deposit(parse_near!("0.0125 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    // storage deposit for Roketo FT
    let res = dao_account
        .call(worker, token_contract.id(), "storage_deposit")
        .args_json(json!({
            "account_id": finance_contract.id()
        }))?
        .deposit(parse_near!("0.0125 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    let res = dao_account
        .call(worker, token_contract.id(), "storage_deposit")
        .args_json(json!({
            "account_id": streaming_contract.id()
        }))?
        .deposit(parse_near!("0.0125 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    Ok((
        streaming_contract,
        finance_contract,
        token_contract,
        wrap_contract,
    ))
}

#[tokio::test]
async fn test_init_contracts() -> anyhow::Result<()> {
    // let initial_balance = U128::from(parse_near!("10000 N"));
    let worker = workspaces::sandbox().await?;
    // TODO: make available through pattern-matching for `wrap.testnet` in the fixture
    // let worker = workspaces::testnet().await?;

    let dao_account = worker.dev_create_account().await?;

    let (streaming_contract, _finance_contract, _token_contract, _wrap_contract) =
        init(&worker, &dao_account).await?;

    let res = streaming_contract
        .call(&worker, "get_dao")
        .args_json(json!({ "account_id": &dao_account.id() }))?
        .view()
        .await?
        .json::<Dao>()?;
    println!("Result of the init command: {:?}", res);

    Ok(())
}

#[tokio::test]
async fn test_init_stream() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let dao_account = worker.dev_create_account().await?;

    let (streaming_contract, _finance_contract, _token_contract, wrap_contract) =
        init(&worker, &dao_account).await?;

    let sender_account = dao_account
        .create_subaccount(&worker, "sender")
        // .initial_balance(parse_near!("0.40 N"))
        .initial_balance(parse_near!("10.00 N"))
        .transact()
        .await?
        .into_result()?;

    let receiver_account = dao_account
        .create_subaccount(&worker, "receiver")
        // .initial_balance(parse_near!("0.01 N"))
        .initial_balance(parse_near!("0.50 N"))
        .transact()
        .await?
        .into_result()?;

    // make a deposit for subsequent transfer for the `sender` account
    let res = sender_account
        .call(&worker, wrap_contract.id(), "near_deposit")
        // .deposit(parse_near!("0.31 N"))
        .deposit(parse_near!("5.51 N"))
        .transact()
        .await?;
    assert!(res.is_success());
    println!("Deposit Result: {:?}", res);

    // check the wrapped balance of the `sender`
    let res = wrap_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": sender_account.id()}))?
        .view()
        .await?
        .json::<U128>()?;
    // assert_eq!(res, U128::from(parse_near!("0.30875 N"))); // the deposit for `sender` on `wrap` after comission
    println!("Sender Balance Result: {:?}", res);

    // launching Roketo stream
    let res = sender_account
        .call(&worker, wrap_contract.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": streaming_contract.id(),
            "amount": "300000000000000000000000",
            "memo":
            "Roketo transfer",
            "msg": format!("{{\"Create\":{{\"request\":{{\"balance\":\"200000000000000000000000\",\"owner_id\":\"{}\",\"receiver_id\":\"{}\",\"token_name\":\"{}\",\"tokens_per_sec\":\"25000000000000000000000\",\"is_locked\":false,\"is_auto_start_enabled\":true,\"description\":\"{{\\\"c\\\":\\\"test\\\"}}\"}}}}}}",
                           sender_account.id(), receiver_account.id(), wrap_contract.id()),
        }))?
        .deposit(ONE_YOCTO)
        .gas(200000000000000)
        .transact()
        .await?;
    assert!(res.is_success());

    sleep(Duration::from_secs(5)).await; // sleep 5 seconds

    // inspect sender account state
    let res = dao_account
        .call(&worker, streaming_contract.id(), "get_account")
        .args_json(json!({"account_id": sender_account.id()}))?
        .view()
        .await?
        .json::<AccountView>()?;
    println!("Sender Account Inspection: {:?}", res);

    // parse the stream Id
    let stream_id_as_hash = res.last_created_stream.unwrap();
    let stream_id = String::from(&stream_id_as_hash);
    println!("Stream Id: {}", stream_id);

    // check the stream status
    let res = dao_account
        .call(&worker, streaming_contract.id(), "get_stream")
        .args_json(json!({ "stream_id": stream_id }))?
        .view()
        .await?
        .json::<Stream>()?;
    println!("Stream Details: {:#?}", res);
    assert_eq!(res.status, StreamStatus::Active);

    Ok(())
}
