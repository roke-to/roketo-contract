pub use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FT_METADATA_SPEC,
};
pub use near_sdk::json_types::U128;
pub use near_sdk::serde_json::json;
pub use near_sdk::{
    env,
    serde_json,
    Balance,
    ONE_YOCTO, //    AccountId,
};
pub use near_units::parse_near;
use std::str;
pub use workspaces::prelude::*;
pub use workspaces::{network::Sandbox, sandbox, Account, AccountId, Contract, Worker};

pub use streaming::{
    AccountView, ContractError, CreateRequest, SafeFloat, Token, TransferCallRequest, ONE_TERA,
    ROKE_TOKEN_DECIMALS, STORAGE_NEEDS_PER_STREAM,
};

const FINANCE_WASM_BYTES: &[u8] = include_bytes!("../res/finance.wasm");
const ROKE_TOKEN_WASM_BYTES: &[u8] = include_bytes!("../res/roke_token.wasm");
const STREAMING_WASM_BYTES: &[u8] = include_bytes!("../res/streaming.wasm");

const FUNGIBLE_TOKEN_WASM_BYTES: &[u8] = include_bytes!("../tests/fungible_token.wasm");

pub type Gas = u64; // Gas is useless in sdk 4.0.0

pub const T_GAS: Gas = 1_000_000_000_000;
pub const DEFAULT_GAS: Gas = 15 * T_GAS;

pub fn to_yocto(value: &str) -> u128 {
    // this code is from near_sdk_sim. I inserted it here because I cannot use this library
    let vals: Vec<_> = value.split('.').collect();
    let part1 = vals[0].parse::<u128>().unwrap() * 10u128.pow(24);
    if vals.len() > 1 {
        let power = vals[1].len() as u32;
        let part2 = vals[1].parse::<u128>().unwrap() * 10u128.pow(24 - power);
        part1 + part2
    } else {
        part1
    }
}

pub struct Env {
    pub worker: Worker<Sandbox>,
    pub near: Account,
    pub roketo: Account,
    pub dao: Account,
    pub streaming: Contract,
    pub finance: Contract,
    pub roketo_token: Contract,
}

pub struct Tokens {
    pub wnear_simple: Contract,
}

pub struct Users {
    pub alice: Account,
    pub charlie: Account,
}

pub async fn storage_deposit(
    worker: &Worker<Sandbox>,
    user: &Account,
    contract_id: &AccountId,
    account_id: &AccountId,
    attached_deposit: Balance,
) -> anyhow::Result<()> {
    user.call(&worker, &contract_id, "storage_deposit")
        .args_json(json!({ "account_id": account_id }))?
        .deposit(attached_deposit)
        .gas(DEFAULT_GAS)
        .transact()
        .await?;
    Ok(())
}

pub async fn ft_storage_deposit(
    worker: &Worker<Sandbox>,
    user: &Account,
    token_account_id: &AccountId,
    account_id: &AccountId,
) -> anyhow::Result<()> {
    storage_deposit(
        worker,
        user,
        token_account_id,
        account_id,
        125 * env::STORAGE_PRICE_PER_BYTE,
    )
    .await?;
    Ok(())
}

pub fn convert_account_id(account_id: &AccountId) -> near_sdk::AccountId {
    let tmp = account_id.as_bytes();
    let s = str::from_utf8(&tmp).unwrap();
    near_sdk::AccountId::new_unchecked(s.to_string())
}

impl Env {
    pub async fn init() -> anyhow::Result<Self> {
        let worker = workspaces::sandbox().await?;
        let near = worker.root_account(); // name will be "near" in mainnet
        let roketo = near
            .create_subaccount(&worker, "r-v2")
            .initial_balance(to_yocto("100000000"))
            .transact()
            .await?
            .into_result()?; // name will be r-v2.near
        let streaming = roketo
            .create_subaccount(&worker, "streaming")
            .initial_balance(to_yocto("10000000"))
            .transact()
            .await?
            .into_result()?; // name will be streaming.r-v2.near
        let finance = roketo
            .create_subaccount(&worker, "finance")
            .initial_balance(to_yocto("10000000"))
            .transact()
            .await?
            .into_result()?; // name will be finance.r-v2.near
        let roketo_token = roketo
            .create_subaccount(&worker, "token")
            .initial_balance(to_yocto("10000000"))
            .transact()
            .await?
            .into_result()?; // name will be token.r-v2.near
        let dao = roketo
            .create_subaccount(&worker, "dao")
            .initial_balance(to_yocto("10000000"))
            .transact()
            .await?
            .into_result()?; // name will be dao.r-v2.near
        let streaming = streaming
            .deploy(&worker, STREAMING_WASM_BYTES)
            .await?
            .into_result()?;
        let finance = finance
            .deploy(&worker, FINANCE_WASM_BYTES)
            .await?
            .into_result()?;
        let roketo_token = roketo_token
            .deploy(&worker, ROKE_TOKEN_WASM_BYTES)
            .await?
            .into_result()?;

        streaming
            .call(&worker, "new")
            .args_json(json!({
                "dao_id": dao.id(),
                "finance_id": finance.id(),
                "utility_token_id": roketo_token.id(),
                "utility_token_decimals": ROKE_TOKEN_DECIMALS,
            }))?
            .transact()
            .await?; // In the old code in this place it was deploy with init method "new". I didn't find such method here, so just did a call. You need to fix this place

        roketo_token
            .call(&worker, "new")
            .args_json("")?
            .transact()
            .await?; // In the old code in this place it was deploy with init method "new". I didn't find such method here, so just did a call. You need to fix this place

        finance
            .call(&worker, "new")
            .args_json(json!({
                "streaming_account_id": streaming.id(),
            }))?
            .transact()
            .await?; // In the old code in this place it was deploy with init method "new". I didn't find such method here, so just did a call. You need to fix this place

        ft_storage_deposit(&worker, &near, &roketo_token.id(), &near.id()).await?;
        ft_storage_deposit(&worker, &near, &roketo_token.id(), &streaming.id()).await?;
        ft_storage_deposit(&worker, &near, &roketo_token.id(), &finance.id()).await?;

        Ok(Self {
            worker,
            near,
            roketo,
            dao,
            streaming,
            finance,
            roketo_token,
        })
    }

    pub async fn setup_assets(&self, tokens: &Tokens) -> anyhow::Result<()> {
        let roketo_token = Token {
            account_id: convert_account_id(self.roketo_token.id()),
            is_payment: true,
            commission_on_create: d(10, 18),
            commission_coef: SafeFloat { val: 1, pow: -4 }, // 0.01%
            commission_on_transfer: d(10, 17),
            storage_balance_needed: 125 * env::STORAGE_PRICE_PER_BYTE,
            gas_for_ft_transfer: near_sdk::Gas(10 * ONE_TERA),
            gas_for_storage_deposit: near_sdk::Gas(10 * ONE_TERA),
        };
        self.streaming
            .call(&self.worker, "dao_update_token")
            .args_json(json!({
                "token": roketo_token,
            }))?;

        let wnear_token = Token {
            account_id: convert_account_id(tokens.wnear_simple.id()),
            is_payment: true,
            commission_on_create: d(1, 23), // 0.1 token
            commission_coef: SafeFloat { val: 4, pow: -3 }, // 0.4%
            commission_on_transfer: d(1, 22),
            storage_balance_needed: 125 * env::STORAGE_PRICE_PER_BYTE,
            gas_for_ft_transfer: near_sdk::Gas(10 * ONE_TERA),
            gas_for_storage_deposit: near_sdk::Gas(10 * ONE_TERA),
        };
        self.streaming
            .call(&self.worker, "dao_update_token")
            .args_json(json!({
                "token": wnear_token,
            }))?;
        Ok(())
    }

    pub async fn contract_ft_transfer_call(
        &self,
        token: &Contract,
        user: &Account,
        amount: Balance,
        msg: &str,
    ) -> anyhow::Result<U128> {
        let num: U128 = user
            .call(&self.worker, token.id(), "ft_transfer_call")
            .args_json(json!({
                "receiver_id": self.streaming.id(),
                "amount": U128::from(amount),
                "msg": msg,
            }))?
            .view()
            .await?
            .json()?;
        Ok(num)
    }

    pub async fn mint_ft(
        &self,
        token: &Contract,
        receiver: &Account,
        amount: Balance,
    ) -> anyhow::Result<()> {
        let caller = if token.id() == self.roketo_token.id() {
            &self.roketo
        } else {
            &self.near
        };
        caller
            .call(&self.worker, token.id(), "ft_transfer")
            .args_json(&json!({
                "receiver_id": receiver.id(),
                "amount": U128::from(amount),
            }))?;
        Ok(())
    }

    pub async fn mint_tokens(
        &self,
        tokens: &Tokens,
        user: &Account,
        amount: Balance,
    ) -> anyhow::Result<()> {
        ft_storage_deposit(&self.worker, user, &tokens.wnear_simple.id(), &user.id()).await?;
        ft_storage_deposit(&self.worker, user, &self.roketo_token.id(), &user.id()).await?;

        if amount > 0 {
            self.mint_ft(&tokens.wnear_simple, user, d(amount, 24))
                .await?;
            self.mint_ft(&self.roketo_token, user, d(amount, 18))
                .await?;
        }
        Ok(())
    }

    pub async fn get_near_balance(&self, user: &Contract) -> anyhow::Result<u128> {
        let amount = user.view_account(&self.worker).await?.balance;
        Ok(amount)
    }

    pub async fn get_balance(&self, token: &Contract, user: &Contract) -> anyhow::Result<u128> {
        let string_value = self
            .near
            .call(&self.worker, token.id(), "ft_balance_of")
            .args_json(&json!({
                "account_id": &user.id(),
            }))?
            .view()
            .await?
            .json::<String>()?;
        Ok(u128::from_str_radix(&string_value[..], 10).unwrap())
    }

    pub async fn create_stream_ext_err(
        &self,
        owner: &Account,
        receiver: &Account,
        token: &Contract,
        amount: Balance,
        tokens_per_sec: Balance,
        description: Option<String>,
        cliff_period_sec: Option<u32>,
        is_auto_start_enabled: Option<bool>,
        is_expirable: Option<bool>,
        is_locked: Option<bool>,
    ) -> anyhow::Result<U128> {
        let tokens_per_sec = U128(tokens_per_sec);
        let something = TransferCallRequest::Create {
            request: CreateRequest {
                owner_id: convert_account_id(owner.id()),
                receiver_id: convert_account_id(receiver.id()),
                tokens_per_sec,
                description,
                cliff_period_sec,
                is_auto_start_enabled,
                is_expirable,
                is_locked,
            },
        };
        let ans = self
            .contract_ft_transfer_call(
                &token,
                &owner,
                amount,
                &serde_json::to_string(&something).unwrap(),
            )
            .await
            .unwrap();
        Ok(ans)
    }
}

pub async fn init_token(e: &Env, token_account_id: &str, decimals: u8) -> anyhow::Result<Contract> {
    let token_account_id: AccountId = token_account_id.parse().unwrap();
    let contract = e.worker.dev_deploy(FUNGIBLE_TOKEN_WASM_BYTES).await?;
    let _token = contract // you need to check it. There is a bug
        .call(&e.worker, "new")
        .args_json(json!({ "owner_id": e.near.id(),
            "total_supply": U128::from(10u128.pow((10 + decimals) as _)),
            "metadata": FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: token_account_id.to_string(),
                symbol: token_account_id.to_string(),
                icon: None,
                reference: None,
                reference_hash: None,
                decimals: decimals,
            }}
        ))?
        .transact()
        .await?
        .json()?;

    ft_storage_deposit(&e.worker, &e.near, &token_account_id, &e.streaming.id()).await?;
    ft_storage_deposit(&e.worker, &e.near, &token_account_id, &e.finance.id()).await?;
    Ok(contract.into())
}

impl Tokens {
    pub async fn init(e: &Env) -> anyhow::Result<Self> {
        Ok(Self {
            wnear_simple: init_token(e, "wnear_simple.near", 24).await?,
        })
    }
}

impl Users {
    pub async fn init(e: &Env) -> anyhow::Result<Self> {
        Ok(Self {
            alice: e
                .near
                .create_subaccount(&e.worker, "alice")
                .initial_balance(to_yocto("10000"))
                .transact()
                .await?
                .into_result()?,
            charlie: e
                .near
                .create_subaccount(&e.worker, "charlie")
                .initial_balance(to_yocto("10000"))
                .transact()
                .await?
                .into_result()?,
        })
    }
}

pub fn d(value: Balance, decimals: u8) -> Balance {
    value * 10u128.pow(decimals as _)
}

pub async fn basic_setup() -> anyhow::Result<(Env, Tokens, Users)> {
    let e = Env::init().await?;
    let tokens = Tokens::init(&e).await?;
    e.setup_assets(&tokens).await?;

    let users = Users::init(&e).await?;
    e.mint_tokens(&tokens, &users.alice, 1000000).await?;

    e.mint_tokens(&tokens, &users.charlie, 0).await?;

    Ok((e, tokens, users))
}
