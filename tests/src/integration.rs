pub use near_sdk::serde_json::json;
use near_units::parse_near;
use near_sdk::{serde_json::to_string, ONE_YOCTO, Balance};
use near_sdk::json_types::U128;
use streaming::{Stream, StreamStatus, AccountView};
use anyhow::Result;

use crate::environment::Environment;
use crate::environment::methods::get_near_balance;
use crate::environment::setup_integration::ExtIntegration;

use common::m_e_n;

#[tokio::test]
async fn test_env_balance() -> Result<()> {
    let b = m_e_n(13, 2);
    assert_eq!(Balance::from(1300u128), b);

    Ok(())
}

#[tokio::test]
async fn test_env_dummy() -> Result<()> {
    let env: Environment<()> = Environment::new().await?;

    let b = get_near_balance(env.streaming.as_account()).await?;
    println!("streaming contract balance: {}", b);
    Ok(())
}

#[tokio::test]
async fn test_env_integration() -> Result<()> {
    let mut env: Environment<ExtIntegration> = Environment::new().await?;

    env.extend_env().await?; //

    assert!(env.extras.is_some());

    Ok(())
}

#[tokio::test]
async fn test_env_stream_sanity() -> Result<()> {
    let mut env: Environment<ExtIntegration> = Environment::new().await?;

    env.extend_env().await?;

    let amount = m_e_n(101, 23);
    let velocity = m_e_n(1, 23);
    let users = env.extras.as_ref().unwrap().users.clone();
    let alice = users.get("alice").unwrap();
    let bob = users.get("bob").unwrap();
    let fts = env.fungible_tokens.clone();
    let wrap_near = fts.get("wrap.testnet").unwrap();

    let stream_id = env
        .create_stream(alice, bob, wrap_near.as_account(), amount, velocity)
        .await?;

    let stream = env.get_stream(&stream_id).await?;
    assert_eq!(stream.status, StreamStatus::Active);
    println!("Stream Balance: {:?}", stream.balance);

    Ok(())
}

#[tokio::test]
async fn test_env_init_stream() -> Result<()> {
    let env: Environment<()> = Environment::new().await?;

    let wrap_near = env.fungible_tokens.get("wrap.testnet").unwrap();

    let sender = env
        .dao
        .create_subaccount("sender")
        .initial_balance(parse_near!("10.00 N"))
        .transact()
        .await?
        .into_result()?;

    let receiver = env
        .dao
        .create_subaccount("receiver")
        .initial_balance(parse_near!("0.50 N"))
        .transact()
        .await?
        .into_result()?;

    // make a deposit for subsequent transfer for the `sender` account
    let res = sender
        .call(env.fungible_tokens["wrap.testnet"].id(), "near_deposit")
        // .deposit(parse_near!("0.31 N"))
        .deposit(parse_near!("5.51 N"))
        .transact()
        .await?;
    assert!(res.is_success());
    println!("Deposit Result: {:?}", res);

    // check the wrapped balance of the `sender`
    let res = sender
        .call(wrap_near.id(), "ft_balance_of")
        .args_json(json!({"account_id": sender.id()}))
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
                "owner_id": sender.id(),
                "receiver_id": receiver.id(),
                "token_name": wrap_near.id(),
                "tokens_per_sec": "25000000000000000000000",
                "is_locked": false,
                "is_auto_start_enabled": true,
                "description": to_string(&json!({ "c": "test" })).unwrap(),
            }
        }
    });

    let res = sender
        .call(wrap_near.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": env.streaming.id(),
            "amount": "300000000000000000000000",
            "memo": "Roketo transfer",
            "msg": to_string(&msg).unwrap(),
        }))
        .deposit(ONE_YOCTO)
        .gas(200000000000000)
        .transact()
        .await?;
    assert!(res.is_success());

    env.sandbox.fast_forward(500).await?;

    // inspect sender account state
    let res = env
        .dao
        .call(env.streaming.id(), "get_account")
        .args_json(json!({"account_id": sender.id()}))
        .view()
        .await?
        .json::<AccountView>()?;
    println!("Sender Account Inspection: {:?}", res);

    // parse the stream Id
    let stream_id_as_hash = res.last_created_stream.unwrap();
    let stream_id = String::from(&stream_id_as_hash);
    println!("Stream Id: {}", stream_id);

    // check the stream status
    let res = env
        .dao
        .call(env.streaming.id(), "get_stream")
        .args_json(json!({ "stream_id": stream_id }))
        .view()
        .await?
        .json::<Stream>()?;
    println!("Stream Details: {:#?}", res);
    assert_eq!(res.status, StreamStatus::Active);

    Ok(())
}
