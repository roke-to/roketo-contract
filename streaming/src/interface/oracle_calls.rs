use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn oracle_update_commission_on_create(
        &mut self,
        token_account_id: AccountId,
        commission_on_create: U128,
    ) {
        self.dao.check_oracle(&env::signer_account_id()).unwrap();
        self.dao
            .tokens
            .entry(token_account_id)
            .and_modify(|e| e.commission_on_create = commission_on_create.into());
    }

    #[payable]
    pub fn oracle_update_eth_near_ratio(&mut self, ratio: SafeFloat) {
        self.dao.check_oracle(&env::signer_account_id()).unwrap();

        ratio.assert_safe();
        self.dao.eth_near_ratio = ratio;
    }
}
