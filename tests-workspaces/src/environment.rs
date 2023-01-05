use anyhow::Result;
use log::info;
use tokio::fs::read;
use workspaces::{network::Sandbox, sandbox, Account, AccountId, Contract, Worker};

use crate::{
    args, assert_all_success, FINANCE_WASM_PATH, STREAMING_WASM_PATH, VAULT_WASM_PATH,
    WRAP_NEAR_WASM_PATH,
};

pub struct Environment {
    sandbox: Worker<Sandbox>,
    streaming: Option<Contract>,
    finance: Option<Contract>,
    wrap_near: Option<Contract>,
    vault: Option<Contract>,
}

impl Environment {
    pub async fn new() -> Result<Self> {
        let sandbox = sandbox().await?;
        info!("sandbox initialized");
        Ok(Self {
            sandbox,
            streaming: None,
            finance: None,
            wrap_near: None,
            vault: None,
        })
    }

    pub fn sandbox(&self) -> &Worker<Sandbox> {
        &self.sandbox
    }

    pub fn streaming(&self) -> &Contract {
        self.streaming.as_ref().expect("streaming is not deployed")
    }

    pub fn finance(&self) -> &Contract {
        self.finance.as_ref().expect("finance is not deployed")
    }

    pub fn wrap_near(&self) -> &Contract {
        self.wrap_near.as_ref().expect("wrap near is not deployed")
    }

    pub fn vault(&self) -> &Contract {
        self.vault.as_ref().expect("vault is not deployed")
    }

    pub async fn deploy_streaming(&mut self, wrap_near_id: AccountId) -> Result<()> {
        assert!(self.streaming.is_none(), "streaming already deployed");
        assert!(self.finance.is_none(), "finance already deployed");
        let streaming_wasm = read(STREAMING_WASM_PATH).await?;
        info!("streaming wasm loaded");
        let streaming = self.sandbox.dev_deploy(&streaming_wasm).await?;
        info!("streaming deployed");

        let finance_wasm = read(FINANCE_WASM_PATH).await?;
        let finance = self.sandbox.dev_deploy(&finance_wasm).await?;
        info!("finance deployed");

        let args = args::streaming_new_json(streaming.id(), finance.id());
        let res = streaming
            .call("new")
            .args_json(args)
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert_all_success(&res);
        info!("streaming contract initialized");

        let args = args::finance_new_json(streaming.id());
        let res = finance
            .call("new")
            .args_json(args)
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert_all_success(&res);
        info!("finance contract initialized");
        self.finance = Some(finance);

        let args = args::streaming_dao_update_token_json(&wrap_near_id);
        let res = streaming
            .call("dao_update_token")
            .args_json(args)
            .deposit(1)
            .transact()
            .await
            .unwrap();
        assert_all_success(&res);
        info!("wrap near registered in streaming");
        info!("STREAMING ID: {}", streaming.id());
        self.streaming = Some(streaming);
        Ok(())
    }

    pub async fn deploy_wrap_near(&mut self) -> Result<()> {
        assert!(self.wrap_near.is_none(), "wrap near already deployed");
        let wrap_near_wasm = read(WRAP_NEAR_WASM_PATH).await?;
        info!("wrap near wasm loaded");
        let wrap_near = self.sandbox.dev_deploy(&wrap_near_wasm).await?;
        info!("wrap near deployed");
        let res = wrap_near.call("new").transact().await?;
        info!("wrap near new called");
        assert_all_success(&res);
        info!("WRAP NEAR ID: {}", wrap_near.id());
        self.wrap_near = Some(wrap_near);
        Ok(())
    }

    pub async fn deploy_vault(&mut self) -> Result<()> {
        assert!(self.vault.is_none(), "vault already deployed");
        let vault_wasm = read(VAULT_WASM_PATH).await?;
        let vault = self.sandbox.dev_deploy(&vault_wasm).await?;
        self.vault = Some(vault);
        Ok(())
    }

    pub async fn wrap_near_register(&self, account: &Account) -> Result<()> {
        let args = args::wrap_near_storage_deposit_json(account.id());
        let res = account
            .call(self.wrap_near().id(), "storage_deposit")
            .args_json(args)
            .deposit(12_500_000_000_000_000_000_000)
            .transact()
            .await?;
        assert_all_success(&res);
        assert_all_success(&res);
        info!("account {} registered", account.id());

        Ok(())
    }

    pub async fn wrap_near_deposit(&self, to: &Account, amount: u128) -> Result<()> {
        let res = to
            .call(self.wrap_near().id(), "near_deposit")
            .deposit(amount)
            .transact()
            .await?;
        assert_all_success(&res);
        Ok(())
    }

    pub async fn wrap_near_ft_transfer_call(&self, sender: &Account) -> Result<()> {
        let args = args::wrap_near_ft_transfer_call_json(
            self.streaming().id(),
            10u128.pow(23),
            sender.id(),
            self.vault().id(),
        );
        let res = sender
            .call(self.wrap_near().id(), "ft_transfer_call")
            .args_json(args)
            .deposit(1)
            .max_gas()
            .transact()
            .await?;
        info!("wrap near ft_transfer_call called");
        assert_all_success(&res);
        Ok(())
    }
}
