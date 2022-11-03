pub use near_sdk::serde_json::json;
use near_units::parse_near;
use near_sdk::{serde_json::to_string, ONE_YOCTO};
use near_sdk::json_types::U128;
use streaming::{Stream, StreamStatus, AccountView};
use anyhow::Result;

use crate::environment::setup_for_vault::ExtVault;
use crate::environment::Environment;

#[tokio::test]
async fn test_env_init_with_vault() -> Result<()> {
    let mut env: Environment<ExtVault> = Environment::new().await?;
    println!("\n<--- base test environment initialized --->\n");

    env.add_vault_to_env().await?;
    println!("\n<--- augmented environment with benefits_vault setup --->\n");

    assert!(env.extras.is_some());

    Ok(())
}
