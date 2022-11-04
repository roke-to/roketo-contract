pub use near_sdk::serde_json::json;
use near_units::parse_near;
use near_sdk::{serde_json::to_string, ONE_YOCTO};
use near_sdk::json_types::U128;
use streaming::{Stream, StreamStatus, AccountView};
use anyhow::Result;

use crate::environment::setup_for_vault::ExtVault;
use crate::environment::{Environment, VAULT_REPLENISH_CALLBACK, VAULT_REPLENISH_ARGS};

#[tokio::test]
async fn test_env_init_with_vault() -> Result<()> {
    let mut env: Environment<ExtVault> = Environment::new().await?;
    println!("\n<--- base test environment initialized --->\n");

    env.add_vault_to_env().await?;
    println!("\n<--- augmented environment with benefits_vault setup --->\n");

    assert!(env.extras.is_some());

    env.add_replenisher().await?;
    let replenishers = env.view_replenishers().await?.expect("must be some");

    assert!(!replenishers.is_empty(), "replenisher wasn't added");
    println!("replenishers are not empty");

    assert_eq!(
        replenishers[0].contract_id().as_str(),
        env.extras.unwrap().issuer.id().as_str(),
        "issuer must be registered as replenisher"
    );
    println!("replenisher contract id is correct");

    assert_eq!(
        replenishers[0].callback(),
        VAULT_REPLENISH_CALLBACK,
        "wrong callback"
    );
    println!("replenisher callback is correct");

    assert_eq!(replenishers[0].args(), VAULT_REPLENISH_ARGS, "wrong args");
    println!("replenisher args are correct");

    Ok(())
}
