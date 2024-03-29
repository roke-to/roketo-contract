use std::collections::HashMap;

mod dao;
mod interface;

pub use crate::dao::*;

pub use common::*;

use near_contract_standards::fungible_token::events::{FtBurn, FtMint};
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, require, AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault,
    Promise, PromiseOrValue, Timestamp, ONE_YOCTO,
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,

    pub dao: Dao,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    FungibleToken,
    Metadata,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(dao_id: AccountId, unlock_price: Balance) -> Self {
        require!(!env::state_exists(), "Already initialized");
        let metadata = FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: ROKE_TOKEN_NAME.to_string(),
            symbol: ROKE_TOKEN_SYMBOL.to_string(),
            icon: Some(ROKE_TOKEN_SVG_ICON.to_string()),
            reference: None,
            reference_hash: None,
            decimals: ROKE_TOKEN_DECIMALS,
        };
        metadata.assert_valid();
        let mut contract = Self {
            token: FungibleToken::new(StorageKey::FungibleToken),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            dao: Dao::new(dao_id, unlock_price),
        };
        contract
            .token
            .internal_register_account(&contract.dao.dao_id);
        contract
    }

    fn mint(&mut self, amount: Balance, memo: Option<&str>) {
        self.token.internal_deposit(&self.dao.dao_id, amount);
        FtMint {
            owner_id: &self.dao.dao_id,
            amount: &amount.into(),
            memo,
        }
        .emit();
    }

    fn burn(&mut self, amount: Balance) {
        self.token
            .internal_withdraw(&env::predecessor_account_id(), amount);
        FtBurn {
            owner_id: &env::predecessor_account_id(),
            amount: &amount.into(),
            memo: None,
        }
        .emit();
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    use super::*;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(1), 0);
        contract.mint(ROKE_TOKEN_TOTAL_SUPPLY, Some("init minting"));
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, ROKE_TOKEN_TOTAL_SUPPLY);
        assert_eq!(
            contract.ft_balance_of(accounts(1)).0,
            ROKE_TOKEN_TOTAL_SUPPLY
        );
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(2), 0);
        contract.mint(ROKE_TOKEN_TOTAL_SUPPLY, None);
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = ROKE_TOKEN_TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(
            contract.ft_balance_of(accounts(2)).0,
            (ROKE_TOKEN_TOTAL_SUPPLY - transfer_amount)
        );
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }

    #[test]
    fn test_multiple_mint() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(1), 0);
        contract.mint(12345, None);
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, 12345);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, 12345);
        testing_env!(context.is_view(false).build());
        contract.mint(ROKE_TOKEN_TOTAL_SUPPLY / 3, None);
        testing_env!(context.is_view(true).build());
        assert_eq!(
            contract.ft_total_supply().0,
            12345 + ROKE_TOKEN_TOTAL_SUPPLY / 3
        );
        assert_eq!(
            contract.ft_balance_of(accounts(1)).0,
            12345 + ROKE_TOKEN_TOTAL_SUPPLY / 3
        );
    }

    #[test]
    fn test_burn() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(1), 0);
        contract.mint(12345, None);
        contract.burn(345);
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, 12000);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, 12000);
        testing_env!(context.is_view(false).build());
        contract.burn(2000);
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, 10000);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, 10000);
    }
}
