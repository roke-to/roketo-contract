use std::time::Duration;

use anyhow::Result;
use log::info;

use crate::{environment::Environment, helpers::init_logger};

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
    env.deploy_nft().await?;
    let nft_owner = env.sandbox().dev_create_account().await?;
    env.nft_mint_to(nft_owner.id()).await?;

    // Register accounts in wrap near contract.
    info!("sender");
    env.wrap_near_register(&sender).await?;
    env.wrap_near_register(env.streaming().as_account()).await?;
    env.wrap_near_register(env.finance().as_account()).await?;
    env.wrap_near_register(env.vault().as_account()).await?;
    info!("NFT owner");
    env.wrap_near_register(&nft_owner).await?;

    // Deposit wrap NEAR to the sender account
    env.wrap_near_deposit(&sender, 10u128.pow(24)).await?;
    info!("wrap near near_deposit to sender called");

    // Make ft_transfer_call with subsequent call to vault.
    env.wrap_near_ft_transfer_call(&sender, nft_owner.id())
        .await?;
    info!("wrap near ft_transfer_call called");
    info!("waiting for 5 secs");
    for i in 0..5 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        info!(". . . . {}", 5 - i - 1);
    }
    info!("Done!");

    env.vault_withdraw(&nft_owner).await?;

    let nft_owner_balance = env.wrap_near_ft_balance_of(&nft_owner).await?;
    info!("nft_owner balance: {nft_owner_balance}");
    assert_eq!(nft_owner_balance, 99599999999999999990040);
    Ok(())
}
