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
}
