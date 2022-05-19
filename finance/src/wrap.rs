use crate::*;

#[ext_contract(ext_wrap_near)]
pub trait ExtWrapNear {
    fn near_withdraw(&mut self, amount: U128) -> Promise;
}

#[ext_contract(ext_self)]
pub trait ExtTransferUnwrapped {
    fn on_near_unwrapped(&mut self, account_id: AccountId, amount: U128) -> Promise;
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn on_near_unwrapped(&mut self, account_id: AccountId, amount: U128) -> Promise {
        Promise::new(account_id).transfer(amount.into())
    }
}

impl Contract {
    pub(crate) fn wrap_near_account_id() -> AccountId {
        AccountId::new_unchecked("wrap.near".to_string())
    }
}
