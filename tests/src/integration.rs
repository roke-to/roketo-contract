pub use near_sdk::serde_json::json;
use near_units::parse_near;
use near_sdk::{serde_json::to_string, ONE_NEAR, ONE_YOCTO};
use near_sdk::json_types::U128;
use tokio::time::{sleep, Duration}; //TODO: switch for fast forward
use streaming::{Stream, StreamStatus, AccountView, Stats};
use anyhow::Result;

use crate::environment::Environment;

#[tokio::test]
async fn test_env_init_stream() -> Result<()> {
    let env = Environment::new().await?;
    println!("\n<--- test environment initialized --->\n");

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

    sleep(Duration::from_secs(5)).await; // sleep 5 seconds

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
