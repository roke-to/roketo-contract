use near_sdk::{serde_json::json, serde_json::to_string, Balance, ONE_YOCTO};
use near_sdk::json_types::{U128, Base58CryptoHash};
use near_units::parse_near;

use common::{abbrevs::*, DEFAULT_GAS};
use common::MAX_GAS;

use streaming::{Stream, AccountView, TransferCallRequest, CreateRequest};

use crate::{WRAP_NEAR_TESTNET_ACCOUNT_ID, UTILITY_TOKEN_SUBACCOUNT_ID};
use crate::environment::Environment;
use crate::environment::format_helpers::format_execution_result;

use anyhow::Result;
use std::collections::HashMap;
use workspaces::{
    network::Sandbox, sandbox, Account, AccountId, Contract, Worker, result::ExecutionFinalResult,
};

// Constant parameters used to setup the environment.
const DEFAULT_INITIAL_BALANCE: Balance = parse_near!("10.00 N");
const DEFAULT_STORAGE_DEPOSIT: Balance = parse_near!("0.0125 N");

pub async fn get_near_balance(account: &Account) -> Result<Balance> {
    Ok(account.view_account().await?.balance)
}

pub async fn get_ft_balance(token: &Account, account: &Account) -> Result<Balance> {
    let res = account
        .call(token.id(), FT_BALANCE_RPC_NAME)
        .args_json(json!({
            "account_id": account.id(),
        }))
        .view()
        .await?
        .json::<U128>()?;

    Ok(Balance::from(res))
}

pub async fn ft_storage_deposit(
    user: &Account,
    token_account_id: &AccountId,
    account_id: &AccountId,
) -> Result<ExecutionFinalResult> {
    let res = user
        .call(token_account_id, STORAGE_DEPOSIT_RPC_NAME)
        .args_json(json!({ "account_id": account_id }))
        .deposit(DEFAULT_STORAGE_DEPOSIT)
        .transact()
        .await?;
    assert!(res.is_success());
    println!(
        "\nStorage deposit for {account_id} on {token_account_id} outcome: {}\n",
        format_execution_result(&res)
    );

    Ok(res)
}

pub async fn mint_ft(
    sandbox: &Worker<Sandbox>,
    token_id: &AccountId,
    receiver: &Account,
    amount: u128,
) -> Result<()> {
    let root = sandbox.root_account()?;
    let res = root
        .call(token_id, FT_TRANSFER_RPC_NAME)
        .args_json(json!({
            "receiver_id": receiver.id(),
            "amount": U128::from(amount),
        }))
        .deposit(1)
        .gas(DEFAULT_GAS.into())
        .transact()
        .await?;
    println!("Mint Result: {:?}", res);
    assert!(res.is_success());

    Ok(())
}

impl<Ext> Environment<'_, Ext> {
    pub async fn streaming_get_account(&self, account: &Account) -> Result<AccountView> {
        let view = self
            .dao
            .call(self.streaming.id(), "get_account")
            .args_json(json!({
                "account_id": account.id(),
                "only_if_exist": Some(false),
            }))
            .transact()
            .await?
            .json::<AccountView>()?;

        Ok(view)
    }

    pub async fn streaming_ft_transfer_call(
        &self,
        token: &Account,
        account: &Account,
        amount: Balance,
        msg: &str,
    ) -> Result<ExecutionFinalResult> {
        let res = account
            .call(token.id(), FT_TRANSFER_CALL_RPC_NAME)
            .args_json(json!({
                "receiver_id": self.streaming.id(),
                "amount": U128::from(amount),
                "msg": msg,
            }))
            .deposit(ONE_YOCTO)
            .gas(MAX_GAS.0)
            .transact()
            .await?;
        Ok(res)
    }

    pub async fn create_stream_ext_err(
        &self,
        owner: &Account,
        receiver: &Account,
        token: &Account,
        amount: Balance,
        tokens_per_sec: Balance,
        description: Option<String>,
        cliff_period_sec: Option<u32>,
        is_auto_start_enabled: Option<bool>,
        is_expirable: Option<bool>,
        is_locked: Option<bool>,
    ) -> Result<U128> {
        let tokens_per_sec = U128(tokens_per_sec);
        let res = self
            .streaming_ft_transfer_call(
                &token,
                &owner,
                amount,
                &to_string(&TransferCallRequest::Create {
                    request: CreateRequest {
                        owner_id: near_sdk::AccountId::new_unchecked(
                            owner.id().as_str().to_string(),
                        ),
                        receiver_id: near_sdk::AccountId::new_unchecked(
                            receiver.id().as_str().to_string(),
                        ),
                        tokens_per_sec,
                        description,
                        cliff_period_sec,
                        is_auto_start_enabled,
                        is_expirable,
                        is_locked,
                    },
                })
                .unwrap(),
            )
            .await?
            .json::<U128>()?;
        Ok(res)
    }

    pub async fn create_stream_ext(
        &self,
        owner: &Account,
        receiver: &Account,
        token: &Account,
        amount: Balance,
        tokens_per_sec: Balance,
        description: Option<String>,
        cliff_period_sec: Option<u32>,
        is_auto_start_enabled: Option<bool>,
        is_expirable: Option<bool>,
        is_locked: Option<bool>,
    ) -> Result<Base58CryptoHash> {
        let amount_accepted = self
            .create_stream_ext_err(
                owner,
                receiver,
                token,
                amount,
                tokens_per_sec,
                description,
                cliff_period_sec,
                is_auto_start_enabled,
                is_expirable,
                is_locked,
            )
            .await?;
        assert_eq!(amount_accepted, U128(amount));
        let stream = self.streaming_get_account(owner).await?;
        Ok(stream.last_created_stream.unwrap())
    }

    pub async fn create_stream(
        &self,
        owner: &Account,
        receiver: &Account,
        token: &Account,
        amount: Balance,
        tokens_per_sec: Balance,
    ) -> Result<Base58CryptoHash> {
        let stream_id = self
            .create_stream_ext(
                owner,
                receiver,
                token,
                amount,
                tokens_per_sec,
                None,
                None,
                None,
                None,
                None,
            )
            .await?;
        Ok(stream_id)
    }

    pub async fn get_stream(&self, stream_id: &Base58CryptoHash) -> Result<Stream> {
        let stream: Stream = self
            .streaming
            .call("get_stream")
            .args_json((stream_id,))
            .view()
            .await?
            .json::<Stream>()?;

        Ok(stream)
    }
}
