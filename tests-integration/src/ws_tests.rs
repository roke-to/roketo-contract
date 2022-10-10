use near_sdk::{serde_json::to_string, ONE_NEAR};
use near_sdk::json_types::U128;
use tokio::time::{sleep, Duration};
use streaming::{Stream, StreamStatus, AccountView, Stats};

use crate::ws_setup::*;

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
        .call("get_dao")
        .args_json(json!({ "account_id": &dao_account.id() }))
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
        .create_subaccount("sender")
        // .initial_balance(parse_near!("0.40 N"))
        .initial_balance(parse_near!("10.00 N"))
        .transact()
        .await?
        .into_result()?;

    let receiver_account = dao_account
        .create_subaccount("receiver")
        // .initial_balance(parse_near!("0.01 N"))
        .initial_balance(parse_near!("0.50 N"))
        .transact()
        .await?
        .into_result()?;

    // make a deposit for subsequent transfer for the `sender` account
    let res = sender_account
        .call(wrap_contract.id(), "near_deposit")
        // .deposit(parse_near!("0.31 N"))
        .deposit(parse_near!("5.51 N"))
        .transact()
        .await?;
    assert!(res.is_success());
    println!("Deposit Result: {:?}", res);

    // check the wrapped balance of the `sender`
    let res = wrap_contract
        .call("ft_balance_of")
        .args_json(json!({"account_id": sender_account.id()}))
        .view()
        .await?
        .json::<U128>()?;
    // assert_eq!(res, U128::from(parse_near!("0.30875 N"))); // the deposit for `sender` on `wrap` after comission
    println!("Sender Balance Result: {:?}", res);

    // launching Roketo stream
    let msg = json!({
        "Create": {
            "request": {
                "balance": "200000000000000000000000",
                "owner_id": sender_account.id(),
                "receiver_id": receiver_account.id(),
                "token_name": wrap_contract.id(),
                "tokens_per_sec": "25000000000000000000000",
                "is_locked": false,
                "is_auto_start_enabled": true,
                "description": to_string(&json!({ "c": "test" })).unwrap(),
            }
        }
    });

    let res = sender_account
        .call(wrap_contract.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": streaming_contract.id(),
            "amount": "300000000000000000000000",
            "memo": "Roketo transfer",
            "msg": to_string(&msg).unwrap(),
        }))
        .deposit(ONE_YOCTO)
        .gas(200000000000000)
        .transact()
        .await?;
    assert!(res.is_success());

    sleep(Duration::from_secs(5)).await; // sleep 5 seconds

    // inspect sender account state
    let res = dao_account
        .call(streaming_contract.id(), "get_account")
        .args_json(json!({"account_id": sender_account.id()}))
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
        .call(streaming_contract.id(), "get_stream")
        .args_json(json!({ "stream_id": stream_id }))
        .view()
        .await?
        .json::<Stream>()?;
    println!("Stream Details: {:#?}", res);
    assert_eq!(res.status, StreamStatus::Active);

    Ok(())
}

#[tokio::test]
async fn test_init_unlisted() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let dao_account = worker.dev_create_account().await?;

    let (streaming_contract, _finance_contract, _token_contract, wrap_contract) =
        init_no_wrap_list(&worker, &dao_account).await?;

    let sender_account = dao_account
        .create_subaccount("sender")
        // .initial_balance(parse_near!("0.40 N"))
        .initial_balance(parse_near!("10.00 N"))
        .transact()
        .await?
        .into_result()?;

    let receiver_account = dao_account
        .create_subaccount("receiver")
        // .initial_balance(parse_near!("0.01 N"))
        .initial_balance(parse_near!("0.50 N"))
        .transact()
        .await?
        .into_result()?;

    // make a deposit for subsequent transfer for the `sender` account
    let res = sender_account
        .call(wrap_contract.id(), "near_deposit")
        // .deposit(parse_near!("0.31 N"))
        .deposit(parse_near!("5.51 N"))
        .transact()
        .await?;
    assert!(res.is_success());
    println!("Deposit Result: {:?}", res);

    // check the wrapped balance of the `sender`
    let res = wrap_contract
        .call("ft_balance_of")
        .args_json(json!({"account_id": sender_account.id()}))
        .view()
        .await?
        .json::<U128>()?;
    // assert_eq!(res, U128::from(parse_near!("0.30875 N"))); // the deposit for `sender` on `wrap` after comission
    println!("Sender Balance Result: {:?}", res);

    // NOTE: adding here missing step
    let res = sender_account
        .call(streaming_contract.id(), "account_deposit_near")
        .deposit(ONE_NEAR)
        .gas(200000000000000)
        .transact()
        .await?;
    assert!(res.is_success());

    let res = streaming_contract
        .call("get_dao")
        .args_json(json!({ "account_id": &dao_account.id() }))
        .view()
        .await?
        .json::<Dao>()?;
    println!("DAO for the streaming Acc : {:?}", res);

    // launching Roketo stream
    let msg = json!({
        "Create": {
            "request": {
                "balance": "200000000000000000000000",
                "owner_id": sender_account.id(),
                "receiver_id": receiver_account.id(),
                "token_name": wrap_contract.id(),
                "tokens_per_sec": "25000000000000000000000",
                "is_locked": false,
                "is_auto_start_enabled": true,
                "description": to_string(&json!({ "c": "test" })).unwrap(),
            }
        }
    });

    let res = sender_account
        .call(wrap_contract.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": streaming_contract.id(),
            "amount": "300000000000000000000000",
            "memo": "Roketo transfer",
            "msg": to_string(&msg).unwrap(),
        }))
        .deposit(ONE_YOCTO)
        .gas(200000000000000)
        .transact()
        .await?;
    assert!(res.is_success());
    println!("Stream Creation: {:?}", res);

    sleep(Duration::from_secs(5)).await; // sleep 5 seconds

    // inspect stats
    let res = streaming_contract
        .call("get_stats")
        .view()
        .await?
        .json::<Stats>()?;
    println!("Stats Output : {:?}", res);

    let res = dao_account
        .call(streaming_contract.id(), "get_account_ft")
        .args_json(json!({
            "account_id": sender_account.id(),
            "token_account_id": wrap_contract.id()
        }))
        .view()
        .await?
        .json::<(U128, U128, U128)>()?;
    println!("Get Account FT: {:?}", res);

    // inspect sender account state
    let res = dao_account
        .call(streaming_contract.id(), "get_account")
        .args_json(json!({"account_id": sender_account.id()}))
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
        .call(streaming_contract.id(), "get_stream")
        .args_json(json!({ "stream_id": stream_id }))
        .view()
        .await?
        .json::<Stream>()?;
    println!("Stream Details: {:#?}", res);
    assert_eq!(res.status, StreamStatus::Active);

    Ok(())
}
