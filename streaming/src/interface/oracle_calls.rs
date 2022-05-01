use crate::*;

#[near_bindgen]
impl Contract {
    #[handle_result]
    #[payable]
    pub fn oracle_update_commission_on_create(
        &mut self,
        token_account_id: AccountId,
        commission_on_create: U128,
    ) -> Result<(), ContractError> {
        // Oracle actions may be delegated to 3rd parties.
        // That's why it uses env::predecessor_account_id() here and below.
        self.dao.check_oracle(&env::predecessor_account_id())?;
        self.dao
            .tokens
            .entry(token_account_id)
            .and_modify(|e| e.commission_on_create = commission_on_create.into());
        Ok(())
    }

    #[handle_result]
    #[payable]
    pub fn oracle_update_eth_near_ratio(&mut self, ratio: SafeFloat) -> Result<(), ContractError> {
        self.dao.check_oracle(&env::predecessor_account_id())?;

        ratio.assert_safe();
        self.dao.eth_near_ratio = ratio;
        Ok(())
    }
}
