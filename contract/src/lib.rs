use std::collections::{HashMap, HashSet};
use std::convert::TryInto;

mod account;
mod aurora;
mod dao;
mod err;
mod interface;
mod primitives;
mod stats;
mod stream;
mod stream_ops;
mod token;
mod unit_tests;
mod web4;

pub use crate::account::*;
pub use crate::aurora::*;
pub use crate::dao::*;
pub use crate::err::*;
pub use crate::interface::*;
pub use crate::primitives::*;
pub use crate::stats::*;
pub use crate::stream::*;
pub use crate::stream_ops::*;
pub use crate::token::*;
pub use crate::unit_tests::*;
pub use crate::web4::*;

use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base58CryptoHash, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json;
use near_sdk::{
    assert_one_yocto, env, ext_contract, log, near_bindgen, AccountId, Balance, BorshStorageKey,
    CryptoHash, Gas, PanicOnDefault, Promise, PromiseOrValue, Timestamp, ONE_NEAR, ONE_YOCTO,
};

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Accounts,
    Stats,
    Streams,
    ActiveStreams { account_id: AccountId },
    InactiveStreams { account_id: AccountId },
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub dao: Dao,
    pub accounts: UnorderedMap<AccountId, VAccount>,
    pub streams: UnorderedMap<StreamId, VStream>,
    pub stats: LazyOption<VStats>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(dao_id: AccountId) -> Self {
        Self {
            dao: Dao::new(dao_id),
            accounts: UnorderedMap::new(StorageKey::Accounts),
            streams: UnorderedMap::new(StorageKey::Streams),
            stats: LazyOption::new(StorageKey::Stats, Some(&Stats::default().into())),
        }
    }
}
