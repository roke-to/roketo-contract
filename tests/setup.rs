pub use near_units::parse_near;
//pub use test_log::test;
use std::str;
pub use workspaces::prelude::*;
pub use workspaces::{network::Sandbox, sandbox, Account, AccountId, Contract, Worker};

pub use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FT_METADATA_SPEC,
};
pub use near_sdk::json_types::U128;
pub use near_sdk::serde_json::json;
pub use near_sdk::{
    env,
    serde_json,
    Balance,
    ONE_YOCTO,
    //    AccountId,
};
//use near_sdk_sim::UserAccount;
// use near_sdk_sim::runtime::GenesisConfig;
//use near_sdk_sim::{deploy, init_simulator, UserAccount, ContractAccount, ExecutionResult};

//use streaming::ContractContract as StreamingContract;
pub use streaming::{
    AccountView, ContractError, CreateRequest, SafeFloat, Token, TransferCallRequest, ONE_TERA,
    ROKE_TOKEN_DECIMALS, STORAGE_NEEDS_PER_STREAM,
};

const FINANCE_WASM_BYTES: &[u8] = include_bytes!("../res/finance.wasm");
const ROKE_TOKEN_WASM_BYTES: &[u8] = include_bytes!("../res/roke_token.wasm");
const STREAMING_WASM_BYTES: &[u8] = include_bytes!("../res/streaming.wasm");

const FUNGIBLE_TOKEN_WASM_BYTES: &[u8] = include_bytes!("../tests/fungible_token.wasm");
const WRAP_NEAR_WASM_BYTES: &[u8] = include_bytes!("../tests/wrap_near.wasm");

pub const NEAR: &str = "near";
pub const ROKETO_ID: &str = "r-v2.near";
pub const STREAMING_ID: &str = "streaming.r-v2.near";
pub const FINANCE_ID: &str = "finance.r-v2.near";
pub const ROKE_TOKEN_ID: &str = "token.r-v2.near";
pub const DAO_ID: &str = "dao.r-v2.near";

pub type Gas = u64; // Gas is useless in sdk 4.0.0

pub const T_GAS: Gas = 1_000_000_000_000;
pub const DEFAULT_GAS: Gas = 15 * T_GAS;
pub const MAX_GAS: Gas = 300 * T_GAS;

pub fn to_yocto(value: &str) -> u128 {
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
    pub root: Account,
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
        .transact()
        .await?;
    Ok(())
}

pub fn ft_storage_deposit(
    worker: &Worker<Sandbox>,
    user: &Account,
    token_account_id: &AccountId,
    account_id: &AccountId,
) {
    storage_deposit(
        worker,
        user,
        token_account_id,
        account_id,
        125 * env::STORAGE_PRICE_PER_BYTE,
    );
}

// // . -> root -> near -> roketo -> dao

impl Env {
    pub async fn init() -> anyhow::Result<Self> {
        let worker = workspaces::sandbox().await?;
        //     let mut genesis_config = GenesisConfig::default();
        //     genesis_config.block_prod_time = 0;
        let root = worker.dev_create_account().await?;
        //     let root = init_simulator(Some(genesis_config));
        //     );
        let dao = worker.dev_create_account().await?;
        //     let dao = roketo.create_user(DAO_ID.parse().unwrap(), to_yocto("10000"));
        let near = worker.dev_create_account().await?;
        //     let near = root.create_user(
        //         AccountId::new_unchecked(NEAR.to_string()),
        //         to_yocto("100000000"),
        let roketo = worker.dev_create_account().await?;
        //     let roketo = near.create_user(ROKETO_ID.parse().unwrap(), to_yocto("20000"));
        let dao_id = dao.id();
        let finance_id: AccountId = FINANCE_ID.parse().unwrap();

        //     let streaming = deploy!(
        //         contract: StreamingContract,
        //         contract_id: STREAMING_ID.to_string(),
        //         bytes: &STREAMING_WASM_BYTES,
        //         signer_account: roketo,
        //         deposit: to_yocto("30"),
        //         gas: DEFAULT_GAS,
        //         init_method: new(
        //             dao_id,
        //             finance_id,
        //             ROKE_TOKEN_ID.parse().unwrap(),
        //             ROKE_TOKEN_DECIMALS
        //         )
        //     );

        let finance = worker.dev_deploy(FINANCE_WASM_BYTES).await?;
        let streaming = worker.dev_deploy(STREAMING_WASM_BYTES).await?;
        let roketo_token = worker.dev_deploy(ROKE_TOKEN_WASM_BYTES).await?;
        // roke_token.call(&worker, "new").args_json()
        //     ,
        //     ROKE_TOKEN_ID.parse().unwrap(),
        //     "new",
        //     b"",
        //     to_yocto("10"),
        //     DEFAULT_GAS,
        // );
        //     let roketo_token = roketo.deploy_and_init(
        //         &ROKE_TOKEN_WASM_BYTES,
        //         ROKE_TOKEN_ID.parse().unwrap(),
        //         "new",
        //         b"",
        //         to_yocto("10"),
        //         DEFAULT_GAS,
        //     );

        //     let finance = roketo.deploy_and_init(
        //         &FINANCE_WASM_BYTES,
        //         FINANCE_ID.parse().unwrap(),
        //         "new",
        //         &json!({
        //             "streaming_account_id": streaming.user_account.account_id()
        //         })
        //         .to_string()
        //         .into_bytes(),
        //         to_yocto("10"),
        //         DEFAULT_GAS,
        //     );

        ft_storage_deposit(&worker, &near, &roketo_token.id(), &near.id());
        ft_storage_deposit(&worker, &near, &roketo_token.id(), &streaming.id());
        ft_storage_deposit(&worker, &near, &roketo_token.id(), &finance.id());

        Ok(Self {
            worker,
            root,
            near,
            roketo,
            dao,
            streaming, // streaming,
            finance,   // finance,
            roketo_token,
        })
    }

    pub async fn setup_assets(
        &self,
        tokens: &Tokens,
    ) -> anyhow::Result<()> {
        let tmp = self.roketo_token.id().as_bytes();
        let s = str::from_utf8(&tmp).unwrap();
        let account_id = near_sdk::AccountId::new_unchecked(s.to_string());
        let token = Token {
            account_id: account_id,
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
                "token": token,
            }))?;

        let tmp = tokens.wnear_simple.id().as_bytes();
        let s = str::from_utf8(&tmp).unwrap();
        let account_id = near_sdk::AccountId::new_unchecked(s.to_string());
        let token = Token {
            account_id: account_id,
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
                "token": token,
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
        let num: U128 = self
            .streaming
            .call(&self.worker, "ft_transfer_call")
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
        ft_storage_deposit(&self.worker, user, &tokens.wnear_simple.id(), &user.id());
        ft_storage_deposit(&self.worker, user, &self.roketo_token.id(), &user.id());

        if amount > 0 {
            self.mint_ft(&tokens.wnear_simple, user, d(amount, 24));
            self.mint_ft(&self.roketo_token, user, d(amount, 18));
        }
        Ok(())
    }

    pub async fn get_near_balance(
        &self,
        user: &Contract,
    ) -> anyhow::Result<u128> {
        let amount = user.view_account(&self.worker).await?.balance;
        Ok(amount)
    }

    pub async fn get_balance(
        &self,
        token: &Contract,
        user: &Contract,
    ) -> anyhow::Result<u128> {
        let tmp = self
            .near
            .call(&self.worker, token.id(), "ft_balance_of")
            .args_json(&json!({
                "account_id": &user.id(),
            }))?
            .view()
            .await?
            .json::<String>()?;
        Ok(u128::from_str_radix(&tmp[..], 10).unwrap())
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
        let ans = self
            .contract_ft_transfer_call(
                &token, &owner, amount,
                "", // &serde_json::to_string(&TransferCallRequest::Create {
                   //     request: CreateRequest {
                   //         owner_id: owner.view_account(streaming),
                   //         receiver_id: receiver.id(),
                   //         tokens_per_sec,
                   //         description,
                   //         cliff_period_sec,
                   //         is_auto_start_enabled,
                   //         is_expirable,
                   //         is_locked,
                   //     },
                   // })
                   // .unwrap(),
            )
            .await
            .unwrap();
        Ok(ans)
    }
}

pub async fn init_token(e: &Env, token_account_id: &str, decimals: u8) -> anyhow::Result<Contract> {
    let token_account_id: AccountId = token_account_id.parse().unwrap();
    let contract = e.worker.dev_deploy(FUNGIBLE_TOKEN_WASM_BYTES).await?;
    let token = contract
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

    ft_storage_deposit(&e.worker, &e.near, &token_account_id, &e.streaming.id());
    ft_storage_deposit(&e.worker, &e.near, &token_account_id, &e.finance.id());
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
                .create_subaccount(&e.worker, "alice.near")
                .initial_balance(to_yocto("10000"))
                .transact()
                .await?
                .into_result()?,
            charlie: e
                .near
                .create_subaccount(&e.worker, "charlie.near")
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

// TODO check balances integrity

pub async fn basic_setup() ->  anyhow::Result<(Env, Tokens, Users)> {
    let e = Env::init().await?;
    let tokens = Tokens::init(&e).await?;
    e.setup_assets(&tokens);

    let users = Users::init(&e).await?;
    e.mint_tokens(&tokens, &users.alice, 1000000);

    e.mint_tokens(&tokens, &users.charlie, 0);

    Ok((e, tokens, users))
}
