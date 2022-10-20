use near_sdk::log;

use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn streaming_ft_transfer(
        &mut self,
        token_account_id: AccountId,
        receiver: AccountId,
        amount: U128,
    ) -> Promise {
        self.check_owner().unwrap();

        self.ft_transfer(token_account_id, receiver, amount.into())
    }

    #[payable]
    pub fn streaming_storage_needs_transfer(&mut self) -> Promise {
        self.check_owner().unwrap();
        log!("Covering storage needs from finance contract");
        Promise::new(self.owner_id.clone()).transfer(STORAGE_NEEDS_PER_STREAM)
    }
}
