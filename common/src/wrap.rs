use crate::*;

#[ext_contract(ext_wrap_near)]
pub trait ExtWrapNear {
    fn near_withdraw(&mut self, amount: U128) -> Promise;
}

pub fn wrap_near_account_id() -> AccountId {
    AccountId::new_unchecked("wrap.near".to_string())
}
