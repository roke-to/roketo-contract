pub use near_units::parse_near;
//pub use test_log::test;
pub use workspaces::prelude::*;
pub use workspaces::{network::Sandbox, sandbox, Account, Contract, Worker};

pub use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FT_METADATA_SPEC,
};
pub use near_sdk::json_types::U128;
pub use near_sdk::serde_json::json;
pub use near_sdk::{env, serde_json, AccountId, Balance, ONE_YOCTO};
use near_sdk_sim::runtime::GenesisConfig;
use near_sdk_sim::{
    deploy, init_simulator, to_yocto, ContractAccount, ExecutionResult, UserAccount,
};
use streaming::ContractContract as StreamingContract;
pub use streaming::{
    AccountView, ContractError, CreateRequest, SafeFloat, Token, TransferCallRequest, ONE_TERA,
    ROKE_TOKEN_DECIMALS, STORAGE_NEEDS_PER_STREAM,
};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    FINANCE_WASM_BYTES => "res/finance.wasm",
    ROKE_TOKEN_WASM_BYTES => "res/roke_token.wasm",
    STREAMING_WASM_BYTES => "res/streaming.wasm",

    FUNGIBLE_TOKEN_WASM_BYTES => "tests/fungible_token.wasm",
    WRAP_NEAR_WASM_BYTES => "tests/wrap_near.wasm",
}

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

pub struct Env {
    pub root: UserAccount,
    pub near: UserAccount,
    pub roketo: UserAccount,
    pub dao: UserAccount,
    pub streaming: ContractAccount<StreamingContract>,
    pub finance: UserAccount,
    pub roketo_token: UserAccount,
}

pub struct Tokens {
    pub wnear_simple: UserAccount,
}

pub struct Users {
    pub alice: UserAccount,
    pub charlie: UserAccount,
}

pub fn storage_deposit(
    user: &UserAccount,
    contract_id: &AccountId,
    account_id: &AccountId,
    attached_deposit: Balance,
) {
    user.call(
        contract_id.clone(),
        "storage_deposit",
        &json!({ "account_id": account_id }).to_string().into_bytes(),
        DEFAULT_GAS,
        attached_deposit,
    )
    .assert_success();
}

pub fn ft_storage_deposit(
    user: &UserAccount,
    token_account_id: &AccountId,
    account_id: &AccountId,
) {
    storage_deposit(
        user,
        token_account_id,
        account_id,
        125 * env::STORAGE_PRICE_PER_BYTE,
    );
}

// . -> root -> near -> roketo -> dao

impl Env {
    pub fn init() -> Self {
        let mut genesis_config = GenesisConfig::default();
        genesis_config.block_prod_time = 0;
        let root = init_simulator(Some(genesis_config));
        let near = root.create_user(
            AccountId::new_unchecked(NEAR.to_string()),
            to_yocto("100000000"),
        );
        let roketo = near.create_user(ROKETO_ID.parse().unwrap(), to_yocto("20000"));
        let dao = roketo.create_user(DAO_ID.parse().unwrap(), to_yocto("10000"));
        let dao_id = dao.account_id();
        let finance_id = FINANCE_ID.parse().unwrap();

        let streaming = deploy!(
            contract: StreamingContract,
            contract_id: STREAMING_ID.to_string(),
            bytes: &STREAMING_WASM_BYTES,
            signer_account: roketo,
            deposit: to_yocto("30"),
            gas: DEFAULT_GAS,
            init_method: new(
                dao_id,
                finance_id,
                ROKE_TOKEN_ID.parse().unwrap(),
                ROKE_TOKEN_DECIMALS
            )
        );

        let roketo_token = roketo.deploy_and_init(
            &ROKE_TOKEN_WASM_BYTES,
            ROKE_TOKEN_ID.parse().unwrap(),
            "new",
            b"",
            to_yocto("10"),
            DEFAULT_GAS,
        );

        let finance = roketo.deploy_and_init(
            &FINANCE_WASM_BYTES,
            FINANCE_ID.parse().unwrap(),
            "new",
            &json!({
                "streaming_account_id": streaming.user_account.account_id()
            })
            .to_string()
            .into_bytes(),
            to_yocto("10"),
            DEFAULT_GAS,
        );

        ft_storage_deposit(&near, &roketo_token.account_id(), &near.account_id());
        ft_storage_deposit(&near, &roketo_token.account_id(), &streaming.account_id());
        ft_storage_deposit(&near, &roketo_token.account_id(), &finance.account_id());

        Self {
            root,
            near,
            roketo,
            dao,
            streaming,
            finance,
            roketo_token,
        }
    }

    pub fn setup_assets(&self, tokens: &Tokens) {
        self.dao
            .function_call(
                self.streaming.contract.dao_update_token(Token {
                    account_id: self.roketo_token.account_id(),
                    is_payment: true,
                    commission_on_create: d(10, 18),
                    commission_coef: SafeFloat { val: 1, pow: -4 }, // 0.01%
                    commission_on_transfer: d(10, 17),
                    storage_balance_needed: 125 * env::STORAGE_PRICE_PER_BYTE,
                    gas_for_ft_transfer: near_sdk::Gas(10 * ONE_TERA),
                    gas_for_storage_deposit: near_sdk::Gas(10 * ONE_TERA),
                }),
                DEFAULT_GAS,
                ONE_YOCTO,
            )
            .assert_success();

        self.dao
            .function_call(
                self.streaming.contract.dao_update_token(Token {
                    account_id: tokens.wnear_simple.account_id(),
                    is_payment: true,
                    commission_on_create: d(1, 23), // 0.1 token
                    commission_coef: SafeFloat { val: 4, pow: -3 }, // 0.4%
                    commission_on_transfer: d(1, 22),
                    storage_balance_needed: 125 * env::STORAGE_PRICE_PER_BYTE,
                    gas_for_ft_transfer: near_sdk::Gas(10 * ONE_TERA),
                    gas_for_storage_deposit: near_sdk::Gas(10 * ONE_TERA),
                }),
                DEFAULT_GAS,
                ONE_YOCTO,
            )
            .assert_success();
    }

    pub fn contract_ft_transfer_call(
        &self,
        token: &UserAccount,
        user: &UserAccount,
        amount: Balance,
        msg: &str,
    ) -> ExecutionResult {
        user.call(
            token.account_id(),
            "ft_transfer_call",
            &json!({
                "receiver_id": self.streaming.account_id(),
                "amount": U128::from(amount),
                "msg": msg,
            })
            .to_string()
            .into_bytes(),
            MAX_GAS,
            1,
        )
    }

    pub fn mint_ft(&self, token: &UserAccount, receiver: &UserAccount, amount: Balance) {
        let caller = if token.account_id() == self.roketo_token.account_id() {
            &self.roketo
        } else {
            &self.near
        };
        caller
            .call(
                token.account_id(),
                "ft_transfer",
                &json!({
                    "receiver_id": receiver.account_id(),
                    "amount": U128::from(amount),
                })
                .to_string()
                .into_bytes(),
                DEFAULT_GAS,
                1,
            )
            .assert_success();
    }

    pub fn mint_tokens(&self, tokens: &Tokens, user: &UserAccount, amount: Balance) {
        ft_storage_deposit(user, &tokens.wnear_simple.account_id(), &user.account_id());
        ft_storage_deposit(user, &self.roketo_token.account_id(), &user.account_id());

        if amount > 0 {
            self.mint_ft(&tokens.wnear_simple, user, d(amount, 24));
            self.mint_ft(&self.roketo_token, user, d(amount, 18));
        }
    }

    pub fn get_near_balance(&self, user: &UserAccount) -> u128 {
        user.account().unwrap().amount
    }

    pub fn get_balance(&self, token: &UserAccount, user: &UserAccount) -> u128 {
        u128::from(
            self.near
                .view(
                    token.account_id(),
                    "ft_balance_of",
                    &json!({
                        "account_id": user.account_id(),
                    })
                    .to_string()
                    .into_bytes(),
                )
                .unwrap_json::<U128>(),
        )
    }

    pub fn create_stream_ext_err(
        &self,
        owner: &UserAccount,
        receiver: &UserAccount,
        token: &UserAccount,
        amount: Balance,
        tokens_per_sec: Balance,
        description: Option<String>,
        cliff_period_sec: Option<u32>,
        is_auto_start_enabled: Option<bool>,
        is_expirable: Option<bool>,
        is_locked: Option<bool>,
    ) -> U128 {
        let tokens_per_sec = U128(tokens_per_sec);
        self.contract_ft_transfer_call(
            &token,
            &owner,
            amount,
            &serde_json::to_string(&TransferCallRequest::Create {
                request: CreateRequest {
                    owner_id: owner.account_id(),
                    receiver_id: receiver.account_id(),
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
        .unwrap_json()
    }
}

pub fn init_token(e: &Env, token_account_id: &str, decimals: u8) -> UserAccount {
    let token_account_id: AccountId = token_account_id.parse().unwrap();
    let token = e.near.deploy_and_init(
        &FUNGIBLE_TOKEN_WASM_BYTES,
        token_account_id.clone(),
        "new",
        &json!({
            "owner_id": e.near.account_id(),
            "total_supply": U128::from(10u128.pow((10 + decimals) as _)),
            "metadata": FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: token_account_id.to_string(),
                symbol: token_account_id.to_string(),
                icon: None,
                reference: None,
                reference_hash: None,
                decimals: decimals,
            }
        })
        .to_string()
        .into_bytes(),
        to_yocto("10"),
        DEFAULT_GAS,
    );

    ft_storage_deposit(&e.near, &token_account_id, &e.streaming.account_id());
    ft_storage_deposit(&e.near, &token_account_id, &e.finance.account_id());
    token
}

impl Tokens {
    pub fn init(e: &Env) -> Self {
        Self {
            wnear_simple: init_token(e, "wnear_simple.near", 24),
        }
    }
}

impl Users {
    pub fn init(e: &Env) -> Self {
        Self {
            alice: e
                .near
                .create_user("alice.near".parse().unwrap(), to_yocto("10000")),
            charlie: e
                .near
                .create_user("charlie.near".parse().unwrap(), to_yocto("10000")),
        }
    }
}

pub fn d(value: Balance, decimals: u8) -> Balance {
    value * 10u128.pow(decimals as _)
}

// TODO check balances integrity

pub fn basic_setup() -> (Env, Tokens, Users) {
    let e = Env::init();
    let tokens = Tokens::init(&e);
    e.setup_assets(&tokens);

    let users = Users::init(&e);
    e.mint_tokens(&tokens, &users.alice, 1000000);

    e.mint_tokens(&tokens, &users.charlie, 0);

    (e, tokens, users)
}
