use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;

mod aurora;
mod err;
mod interface;
mod token;

/*mod account;
mod dao;
mod primitives;
mod stats;
mod stream;
mod stream_ops;
mod token;
mod unit_tests;
mod web4;

pub use crate::account::*;
pub use crate::dao::*;
pub use crate::err::*;
pub use crate::interface::token_calls::*;
pub use crate::interface::views::*;
pub use crate::primitives::*;
pub use crate::stats::*;
pub use crate::stream::*;*/

pub use crate::aurora::*;
pub use crate::err::*;
pub use crate::token::*;

use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base58CryptoHash, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json;
use near_sdk::store::LazyOption;
use near_sdk::{
    assert_one_yocto, env, ext_contract, log, near_bindgen, AccountId, Balance, BorshStorageKey,
    CryptoHash, Gas, PanicOnDefault, Promise, PromiseOrValue, Timestamp, ONE_NEAR, ONE_YOCTO,
};

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Accounts,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub accounts: UnorderedMap<AccountId, String>,
    //pub stats: LazyOption<VStats>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(streaming_account_id: AccountId) -> Self {
        Self {
            owner_id: streaming_account_id,
            accounts: UnorderedMap::new(StorageKey::Accounts),
            //stats: LazyOption::new(StorageKey::Stats, Some(Stats::default().into())),
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
