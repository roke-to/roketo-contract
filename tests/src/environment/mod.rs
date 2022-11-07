mod format_helpers;
pub mod methods;
mod setup;
pub mod setup_integration;
pub mod setup_vault;

// use format_helpers::format_execution_result;

use self::setup::{
    prepare_wrap_near_contract, deploy_streaming_contract, deploy_finance_contract,
    deploy_utility_token_contract, init_roketo_contracts, register_fts_on_streaming,
    add_storage_deposit,
};

use self::setup_integration::{ExtIntegration, prepare_users, mint_tokens};
use self::setup_vault::{
    ExtVault, prepare_external_vault_contract, prepare_issuer_account, prepare_nft_owner_account,
    prepare_external_nft_contract,
};

use crate::{WRAP_NEAR_TESTNET_ACCOUNT_ID, UTILITY_TOKEN_SUBACCOUNT_ID};

use anyhow::Result;
use std::collections::HashMap;
use workspaces::{network::Sandbox, sandbox, Account, Contract, Worker};

/// Struct containing a set of contracts that streaming operations rely upon.
/// Deployed for testing together under the following environment.
#[derive(Clone)]
pub struct Environment<'a, Ext> {
    /// Sandboxed network worker.
    pub sandbox: Worker<Sandbox>,
    /// The DAO account that is in charge of the streaming environment.
    pub dao: Account,
    /// Hashmap collection of FT-contracts used throughout tests.
    pub fungible_tokens: HashMap<&'a str, Contract>,
    /// Roketo Streaming contract.
    pub streaming: Contract,
    /// Roketo Finance contract.
    pub finance: Contract,
    /// Extension fields for complex environments
    /// that include infrastructure for external contract testing.
    pub extras: Option<Ext>,
}

impl<Ext> Environment<'_, Ext> {
    pub async fn new() -> Result<Environment<'static, Ext>> {
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
        fungible_tokens.insert(WRAP_NEAR_TESTNET_ACCOUNT_ID, wrap_near);
        fungible_tokens.insert(UTILITY_TOKEN_SUBACCOUNT_ID, roketo_ft);

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

        println!("\n<--- test environment initialized --->\n");

        Ok(Environment {
            sandbox,
            dao,
            fungible_tokens,
            streaming,
            finance,
            extras: None,
        })
    }
}

impl Environment<'_, ExtIntegration<'_>> {
    pub async fn extend_env(&mut self) -> Result<(), anyhow::Error> {
        let users = tokio::spawn(prepare_users(self.sandbox.clone())).await??;

        mint_tokens(&self.sandbox, &self.fungible_tokens, &users, 10000).await?;

        let extras = ExtIntegration { users };

        self.extras = Some(extras);
        println!("\n<--- environment extended with test user accounts data --->\n");

        Ok(())
    }
}

impl Environment<'static, ExtVault> {
    pub async fn add_vault_to_env(&mut self) -> Result<(), anyhow::Error> {
        let issuer = tokio::spawn(prepare_issuer_account(
            self.sandbox.clone(),
            self.dao.clone(),
            self.fungible_tokens.clone(),
        ));
        let nft_owner = tokio::spawn(prepare_nft_owner_account(
            self.sandbox.clone(),
            self.dao.clone(),
            self.fungible_tokens.clone(),
        ));
        let vault = tokio::spawn(prepare_external_vault_contract(
            self.sandbox.clone(),
            self.dao.clone(),
            self.fungible_tokens.clone(),
        ));
        let nft = tokio::spawn(prepare_external_nft_contract(
            self.sandbox.clone(),
            self.dao.clone(),
            self.fungible_tokens.clone(),
        ));

        let issuer = issuer.await??;
        let nft_owner = nft_owner.await??;
        let vault = vault.await??;
        let nft = nft.await??;

        let extras = ExtVault {
            issuer,
            nft_owner,
            vault,
            nft,
        };
        self.extras = Some(extras);

        Ok(())
    }
}
