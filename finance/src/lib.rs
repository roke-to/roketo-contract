mod interface;
mod transfer;

pub use crate::transfer::*;

use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
#[allow(unused_imports)]
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, near_bindgen, AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault,
    Promise, ONE_YOCTO,
};

pub use common::*;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Accounts,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub accounts: UnorderedMap<AccountId, String>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(streaming_account_id: AccountId) -> Self {
        Self {
            owner_id: streaming_account_id,
            accounts: UnorderedMap::new(StorageKey::Accounts),
        }
    }
}

impl Contract {
    pub(crate) fn check_owner(&self) -> Result<(), ContractError> {
        if env::predecessor_account_id() == self.owner_id {
            Ok(())
        } else {
            Err(ContractError::PredecessorIsNotOwner {
                expected: self.owner_id.clone(),
                received: env::predecessor_account_id(),
            })
        }
    }
}
