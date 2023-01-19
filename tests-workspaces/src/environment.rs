use anyhow::Result;
use log::info;
use near_sdk::{json_types::U128, serde_json::json};
use tokio::fs::read;
use workspaces::{network::Sandbox, sandbox, Account, AccountId, Contract, Worker};

use crate::{
    args, helpers::assert_all_success, FINANCE_WASM_PATH, NFT_TOKEN_ID, NFT_WASM_PATH,
    STREAMING_WASM_PATH, VAULT_WASM_PATH, WRAP_NEAR_WASM_PATH,
};

pub struct Environment {
    sandbox: Worker<Sandbox>,
    streaming: Option<Contract>,
    finance: Option<Contract>,
    wrap_near: Option<Contract>,
    vault: Option<Contract>,
    nft: Option<Contract>,
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
            nft: None,
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

    pub fn nft(&self) -> &Contract {
        self.nft.as_ref().expect("nft is not deployed")
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

        let streaming_cloned = streaming.clone();
        let finance_id = finance.id().clone();
        let finance_cloned = finance.clone();
        let streaming_id = streaming.id().clone();
        tokio::spawn(async move {
            let args = args::streaming_new_json(streaming_cloned.id(), &finance_id);
            let res = streaming_cloned
                .call("new")
                .args_json(args)
                .max_gas()
                .transact()
                .await
                .unwrap();
            assert_all_success(&res);
            info!("-> streaming contract initialized");

            let args = args::finance_new_json(&streaming_id);
            let res = finance_cloned
                .call("new")
                .args_json(args)
                .max_gas()
                .transact()
                .await
                .unwrap();
            assert_all_success(&res);
            info!("-> finance contract initialized");

            let args = args::streaming_dao_update_token_json(&wrap_near_id);
            let res = streaming_cloned
                .call("dao_update_token")
                .args_json(args)
                .deposit(1)
                .transact()
                .await
                .unwrap();
            assert_all_success(&res);
            info!("-> wrap near registered in streaming");
        });

        info!("FINANCE ID: {}", finance.id());
        self.finance = Some(finance);

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
        let wn = wrap_near.clone();
        tokio::spawn(async move {
            let res = wn.call("new").transact().await.unwrap();
            info!("-> wrap near new called");
            assert_all_success(&res);
        });
        info!("WRAP NEAR ID: {}", wrap_near.id());
        self.wrap_near = Some(wrap_near);
        Ok(())
    }

    pub async fn deploy_vault(&mut self) -> Result<()> {
        assert!(self.vault.is_none(), "vault already deployed");
        let vault_wasm = read(VAULT_WASM_PATH).await?;
        let vault = self.sandbox.dev_deploy(&vault_wasm).await?;
        let vault_cloned = vault.clone();
        tokio::spawn(async move {
            let res = vault_cloned.call("new").transact().await.unwrap();
            info!("-> vault new called");
            assert_all_success(&res);
        });
        self.vault = Some(vault);
        Ok(())
    }

    pub async fn deploy_nft(&mut self) -> Result<()> {
        assert!(self.nft.is_none(), "NFT already deployed");
        let nft_wasm = read(NFT_WASM_PATH).await?;
        let nft = self.sandbox.dev_deploy(&nft_wasm).await?;
        info!("nft deployed");
        let nft_cloned = nft.clone();
        tokio::spawn(async move {
            let args = json!({
                "owner_id": nft_cloned.id(),
            });

            let res = nft_cloned
                .call("new_default_meta")
                .args_json(args)
                .transact()
                .await
                .unwrap();
            info!("-> nft new_default_meta called");
            assert_all_success(&res);
        });
        self.nft = Some(nft);
        Ok(())
    }

    pub async fn wrap_near_register(&self, account: &Account) -> Result<()> {
        let wrap_near_id = self.wrap_near().id().clone();
        let account = account.clone();
        tokio::spawn(async move {
            let args = args::wrap_near_storage_deposit_json(account.id());
            let res = account
                .call(&wrap_near_id, "storage_deposit")
                .args_json(args)
                .deposit(12_500_000_000_000_000_000_000)
                .transact()
                .await
                .unwrap();
            assert_all_success(&res);
            info!("-> account {} registered", account.id());
        });

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

    pub async fn wrap_near_ft_transfer_call(
        &self,
        sender: &Account,
        nft_owner: &AccountId,
    ) -> Result<()> {
        let args = args::wrap_near_ft_transfer_call_json(
            self.streaming().id(),
            10u128.pow(23),
            nft_owner,
            self.vault().id(),
            self.nft().id(),
        );
        let res = sender
            .call(self.wrap_near().id(), "ft_transfer_call")
            .args_json(args)
            .deposit(1)
            .max_gas()
            .transact()
            .await?;
        info!("wrap near ft_transfer_call called");
        info!("{:#?}", res);
        assert_all_success(&res);
        Ok(())
    }

    pub async fn wrap_near_ft_balance_of(&self, account: &Account) -> Result<u128> {
        let args = json!({
            "account_id": account.id(),
        });
        let res = self
            .wrap_near()
            .view("ft_balance_of")
            .args_json(args)
            .await?;
        let balance: U128 = res.json()?;
        Ok(balance.0)
    }

    pub async fn vault_withdraw(&self, owner: &Account) -> Result<()> {
        let args = json!({
            "nft_contract_id": self.nft().id(),
            "nft_id": NFT_TOKEN_ID,
            "fungible_token": self.wrap_near().id(),
        });
        let res = owner
            .call(self.vault().id(), "withdraw")
            .args_json(args)
            .deposit(1)
            .max_gas()
            .transact()
            .await?;
        info!("vault withdraw called");
        info!("{:#?}", res);
        assert_all_success(&res);
        Ok(())
    }

    pub async fn nft_mint_to(&self, to: &AccountId) -> Result<()> {
        let token_metadata = json!({
            "title": "Olympus Mons",
            "description": "Tallest mountain in charted solar system",
            "media": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/00/Olympus_Mons_alt.jpg/1024px-Olympus_Mons_alt.jpg",
            "copies": 1
        });
        let args = json!({
            "token_id": NFT_TOKEN_ID,
            "receiver_id": to,
            "token_metadata": token_metadata
        });
        let nft = self.nft().clone();
        tokio::spawn(async move {
            let res = nft
                .call("nft_mint")
                .args_json(args)
                .deposit(8_450_000_000_000_000_000_000)
                .transact()
                .await
                .unwrap();
            info!("-> nft nft_mint called");
            assert_all_success(&res);
        });
        Ok(())
    }
}
