use anyhow::Result;
use log::info;

use crate::environment::Environment;

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

    // Deposit wrap NEAR to the sender account
    env.wrap_near_deposit(&sender, 10u128.pow(24)).await?;
    info!("wrap near near_deposit to sender called");

    // Make ft_transfer_call with subsequent call to vault.
    env.wrap_near_ft_transfer_call(&sender).await?;
    info!("wrap near ft_transfer_call called");
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
