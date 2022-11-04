mod format_helpers;
mod setup;
pub mod setup_for_vault;

use format_helpers::format_execution_result;

use crate::environment::setup::{
    prepare_wrap_near_contract, deploy_streaming_contract, deploy_finance_contract,
    deploy_utility_token_contract, init_roketo_contracts, register_fts_on_streaming,
    add_storage_deposit,
};
use self::setup_for_vault::{
    Replenisher, ExtVault, prepare_external_vault_contract, prepare_issuer_account,
    prepare_nft_owner_account, prepare_external_nft_contract,
};

use crate::{WRAP_NEAR_TESTNET_ACCOUNT_ID, UTILITY_TOKEN_SUBACCOUNT_ID};

use anyhow::Result;
use std::collections::HashMap;
use workspaces::{network::Sandbox, sandbox, Account, Contract, Worker};
use near_sdk::{
    json_types::U128,
    serde_json::{json, to_vec},
};

pub const NFT_TOKEN_ID: &str = "munch_scream";
// pub const NFT_TRANSFER_CALL: &str = "nft_transfer";
pub const VAULT_REPLENISH_CALLBACK: &str = "request_ft";
pub const VAULT_REPLENISH_ARGS: &str = "{arg: \"some arg\"}";
pub const VAULT_ADD_REPLENISHMENT_CALLBACK_CALL: &str = "add_replenishment_callback";
pub const VAULT_VIEW_REPLENISHERS_CALL: &str = "replenishers";

/// Struct containing a set of contracts that streaming operations rely upon.
/// Deployed for testing together under the following environment.
pub struct Environment<Ext> {
    /// Sandboxed network worker.
    pub sandbox: Worker<Sandbox>,
    /// The DAO account that is in charge of the streaming environment.
    pub dao: Account,
    /// Hashmap collection of FT-contracts used throughout tests.
    pub fungible_tokens: HashMap<String, Contract>,
    /// Roketo Streaming contract.
    pub streaming: Contract,
    /// Roketo Finance contract.
    pub finance: Contract,
    /// Extension fields for complex environments
    /// that include infrastructure for external contract testing.
    pub extras: Option<Ext>,
}

impl<Ext> Environment<Ext> {
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
            extras: None,
        })
    }
}

impl Environment<ExtVault> {
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

    pub async fn add_replenisher(&self) -> Result<()> {
        let extras = self.extras.clone().unwrap();

        let args = json!({
            "nft_contract_id": extras.nft.id(),
            "nft_id": NFT_TOKEN_ID,
            "callback": VAULT_REPLENISH_CALLBACK,
            "args": VAULT_REPLENISH_ARGS,
        });
        let res = extras
            .issuer
            .call(extras.vault.id(), VAULT_ADD_REPLENISHMENT_CALLBACK_CALL)
            .args_json(args)
            .deposit(1)
            .transact()
            .await?;

        println!("add replenisher: {}", format_execution_result(&res));

        Ok(())
    }

    pub async fn view_replenishers(&self) -> Result<Option<Vec<Replenisher>>> {
        let extras = self.extras.clone().unwrap();

        let args = to_vec(&json!({
            "nft_contract_id": extras.nft.id(),
            "nft_id": NFT_TOKEN_ID,
        }))?;
        let res = extras
            .issuer
            .view(extras.vault.id(), VAULT_VIEW_REPLENISHERS_CALL, args)
            .await?;

        println!("view replenishers logs: {:?}", res.logs);

        let replenishers = res.json()?;

        Ok(replenishers)
    }
}
