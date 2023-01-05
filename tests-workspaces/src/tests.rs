use anyhow::Result;
use log::info;
use near_sdk::{json_types::U128, serde_json::json};
use tokio::fs::read;

use crate::{environment::Environment, VAULT_WASM_PATH};

#[tokio::test]
async fn test_token_calls_create_stream_call() -> Result<()> {
    init_logger();
    let mut env = Environment::new().await?;
    env.deploy_streaming().await?;
    env.deploy_wrap_near().await?;
    let sender = env.sandbox().dev_create_account().await?;
    info!("sender account created: {}", sender.id());
    let res = sender
        .call(env.wrap_near().id(), "near_deposit")
        .deposit(10u128.pow(24))
        .transact()
        .await?;
    info!("wrap near near_deposit called");
    info!("{res:#?}");
    let args = json!(
        {
            "account_id": env.streaming().id(),
        }
    );

    let res = env
        .streaming()
        .as_account()
        .call(env.wrap_near().id(), "storage_deposit")
        .args_json(args)
        .deposit(12_500_000_000_000_000_000_000)
        .transact()
        .await?;
    info!("wrap near storage_deposit called");
    info!("{res:#?}");
    let vault_wasm = read(VAULT_WASM_PATH).await?;
    let vault = env.sandbox().dev_deploy(&vault_wasm).await?;

    let request = json!({
        "owner_id": sender.id(),
        "receiver_id": vault.id(),
        "tokens_per_sec": "1000",
        "is_auto_start_enabled": true,
    });
    let stream_ids: Vec<String> = vec![];
    let withdraw_args = json!({
        "stream_ids": stream_ids,
    });
    let vault_args = json!({
        "nft_contract_id": "nft.testnet",
        "nft_id": "token_id_0",
        "callback": "withdraw",
        "args": withdraw_args,
    });
    let msg = json!({
        "Create": {
            "request": request,
        },
        "contract": vault.id(),
        "call": "add_replenishment_callback",
        "args": vault_args,
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
    info!("{res:#?}");
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
