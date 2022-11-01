mod format_helpers;
mod setup;

// use format_helpers::format_execution_result;

use crate::environment::setup::{
    prepare_wrap_near_contract, deploy_streaming_contract, deploy_finance_contract,
    deploy_utility_token_contract, init_roketo_contracts, register_fts_on_streaming,
    add_storage_deposit,
};

use crate::{WRAP_NEAR_TESTNET_ACCOUNT_ID, UTILITY_TOKEN_SUBACCOUNT_ID};

use anyhow::Result;
use std::{collections::HashMap};
use workspaces::{network::Sandbox, sandbox, Account, Contract, Worker};

/// Struct containing a set of contracts that streaming operations rely upon.
/// Deployed for testing together under the following environment.
pub struct Environment {
    /// Sandboxed network worker.
    pub sandbox: Worker<Sandbox>,
    /// The DAO account that is in charge of the streaming environment.
    pub dao: Account,
    /// Hashmap collection of FT-contracts used throughout tests.
    pub fungible_tokens: HashMap<String, Contract>,
    /// Roketo Streaming contract
    pub streaming: Contract,
    /// Roketo Finance contract
    pub finance: Contract,
}

impl Environment {
    pub async fn new() -> Result<Self> {
        let sandbox = sandbox().await?;
        println!("sandbox initialized");

        let dao = sandbox.dev_create_account().await?;
        println!("deployed DAO account under: {}", dao.id());

        let wrap_near = tokio::spawn(prepare_wrap_near_contract(sandbox.clone()));
        let roketo_ft = tokio::spawn(deploy_utility_token_contract(dao.clone()));

        let wrap_near = wrap_near.await??;
        let roketo_ft = roketo_ft.await??;
        println!("wrap NEAR token account ready on: {}\n", wrap_near.id());
        println!(
            "Roketo utility fungible token ready on: {}\n",
            roketo_ft.id()
        );
        let mut fungible_tokens = HashMap::new();
        fungible_tokens.insert(WRAP_NEAR_TESTNET_ACCOUNT_ID.to_string(), wrap_near);
        fungible_tokens.insert(UTILITY_TOKEN_SUBACCOUNT_ID.to_string(), roketo_ft);

        let streaming = tokio::spawn(deploy_streaming_contract(dao.clone()));
        let finance = tokio::spawn(deploy_finance_contract(dao.clone()));

        let streaming = streaming.await??;
        let finance = finance.await??;

        let (streaming, finance) = tokio::spawn(init_roketo_contracts(
            dao.clone(),
            streaming.clone(),
            finance.clone(),
            fungible_tokens.clone(),
        ))
        .await??;

        let streaming_handle = tokio::spawn(register_fts_on_streaming(
            dao.clone(),
            streaming.clone(),
            fungible_tokens.clone(),
        ));

        let fungible_tokens = tokio::spawn(add_storage_deposit(
            dao.clone(),
            streaming.clone(),
            finance.clone(),
            fungible_tokens.clone(),
        ));

        let streaming = streaming_handle.await??;
        let fungible_tokens = fungible_tokens.await??;

        Ok(Environment {
            sandbox,
            dao,
            fungible_tokens,
            streaming,
            finance,
        })
    }
}
