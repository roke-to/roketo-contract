pub use near_sdk::serde_json::json;
use near_units::parse_near;
use near_sdk::{serde_json::to_string, ONE_YOCTO};
use near_sdk::json_types::U128;
use streaming::{Stream, StreamStatus, AccountView};
use anyhow::Result;

use crate::environment::setup_vault::ExtVault;
use crate::environment::Environment;

#[tokio::test]
async fn test_env_init_with_vault() -> Result<()> {
    let mut env: Environment<ExtVault> = Environment::new().await?;

    env.add_vault_to_env().await?;
    println!("\n<--- augmented environment with benefits_vault setup --->\n");

    assert!(env.extras.is_some());

    Ok(())
}

#[tokio::test]
async fn test_env_add_replenisher() -> Result<()> {
    let mut env: Environment<ExtVault> = Environment::new().await?;

    env.add_vault_to_env().await?;
    println!("\n<--- augmented environment with benefits_vault setup --->\n");

    assert!(env.extras.is_some());

    env.vault_add_replenisher().await?;

    let replenishers = env
        .vault_view_replenishers()
        .await?
        .expect("must be some, because vault is created");

    assert!(!replenishers.is_empty(), "replenisher wasn't added");
    println!("replenishers are not empty");

    Ok(())
}
