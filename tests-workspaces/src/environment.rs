use anyhow::Result;
use log::info;
use near_sdk::serde_json::json;
use tokio::fs::read;
use workspaces::{network::Sandbox, sandbox, Contract, Worker};

use crate::{STREAMING_WASM_PATH, WRAP_NEAR_WASM_PATH};

pub struct Environment {
    sandbox: Worker<Sandbox>,
    streaming: Option<Contract>,
    wrap_near: Option<Contract>,
}

impl Environment {
    pub async fn new() -> Result<Self> {
        let sandbox = sandbox().await?;
        info!("sandbox initialized");
        Ok(Self {
            sandbox,
            streaming: None,
            wrap_near: None,
        })
    }

    pub fn sandbox(&self) -> &Worker<Sandbox> {
        &self.sandbox
    }

    pub fn streaming(&self) -> &Contract {
        self.streaming.as_ref().expect("streaming is not deployed")
    }

    pub fn wrap_near(&self) -> &Contract {
        self.wrap_near.as_ref().expect("wrap near is not deployed")
    }

    pub async fn deploy_streaming(&mut self) -> Result<()> {
        assert!(self.streaming.is_none(), "streaming already deployed");
        let streaming_wasm = read(STREAMING_WASM_PATH).await?;
        info!("streaming wasm loaded");
        let streaming = self.sandbox.dev_deploy(&streaming_wasm).await?;
        info!("streaming deployed");
        let args = json!({
            "dao_id": streaming.id(),
            "finance_id": "finance.testnet",
            "utility_token_id": "token-roketo.testnet",
            "utility_token_decimals": 18
        });
        let res = streaming
            .call("new")
            .args_json(args)
            .max_gas()
            .transact()
            .await
            .unwrap();
        info!("{res:#?}");
        info!("streaming contract initialized");
        info!("STREAMING ID: {}", streaming.id());
        self.streaming = Some(streaming);
        Ok(())
    }

    pub async fn deploy_wrap_near(&mut self) -> Result<()> {
        let wrap_near_wasm = read(WRAP_NEAR_WASM_PATH).await?;
        info!("wrap near wasm loaded");
        let wrap_near = self.sandbox.dev_deploy(&wrap_near_wasm).await?;
        info!("wrap near deployed");
        let res = wrap_near.call("new").transact().await?;
        info!("wrap near new called");
        info!("{res:#?}");
        info!("WRAP NEAR ID: {}", wrap_near.id());
        self.wrap_near = Some(wrap_near);
        Ok(())
    }
}
