use anyhow::Result;
use log::info;
use near_sdk::{json_types::U128, serde_json::json};

use crate::{assert_all_success, environment::Environment};

#[tokio::test]
async fn test_token_calls_create_stream_call() -> Result<()> {
    init_logger();
    let mut env = Environment::new().await?;
    env.deploy_wrap_near().await?;
    let wrap_near_id = env.wrap_near().id().clone();
    env.deploy_streaming(wrap_near_id).await?;
    env.deploy_vault().await?;
    let sender = env.sandbox().dev_create_account().await?;
    info!("sender account created: {}", sender.id());

    // Register accounts in wrap near contract.
    env.wrap_near_register(&sender).await?;
    env.wrap_near_register(env.streaming().as_account()).await?;
    env.wrap_near_register(env.finance().as_account()).await?;
    env.wrap_near_register(env.vault().as_account()).await?;

    env.wrap_near_deposit(&sender, 10u128.pow(24)).await?;
    info!("wrap near near_deposit to sender called");

    let request = json!({
        "owner_id": sender.id(),
        "receiver_id": env.vault().id(),
        "tokens_per_sec": "1000",
        "is_auto_start_enabled": true,
    });
    let stream_ids: Vec<String> = vec![];
    let withdraw_args = json!({
        "stream_ids": stream_ids,
    })
    .to_string();
    let vault_args = json!({
        "nft_contract_id": "nft.testnet",
        "nft_id": "token_id_0",
        "callback": "withdraw",
        "args": withdraw_args,
    })
    .to_string();
    let msg = json!({
        "CreateCall": {
            "request": request,
            "contract": env.vault().id(),
            "call": "add_replenishment_callback",
            "args": vault_args,
        },
    })
    .to_string();
    let args = json!({
        "receiver_id": env.streaming().id(),
        "amount": U128(10u128.pow(23)),
        "msg": msg,
    });
    let res = sender
        .call(env.wrap_near().id(), "ft_transfer_call")
        .args_json(args)
        .deposit(1)
        .max_gas()
        .transact()
        .await?;
    info!("wrap near ft_transfer_call called");
    assert_all_success(&res);
    todo!()
}

pub fn init_logger() {
    if let Err(e) = env_logger::Builder::new()
        .parse_env("RUST_LOG")
        .format_timestamp(None)
        .format_module_path(false)
        .format_target(false)
        .try_init()
    {
        info!("logger already initialized: {}", e);
    }
}
