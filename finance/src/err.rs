use crate::*;

use near_sdk::FunctionError;

#[derive(BorshDeserialize, BorshSerialize, Serialize, PartialEq, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize))]
#[serde(crate = "near_sdk::serde")]
pub enum ContractError {
    Unknown,
    PredecessorIsNotOwner {
        expected: AccountId,
        received: AccountId,
    },
}

impl FunctionError for ContractError {
    fn panic(&self) -> ! {
        crate::env::panic_str(&format!("{:?}", self))
    }
}
