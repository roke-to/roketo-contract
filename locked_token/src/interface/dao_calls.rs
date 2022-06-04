use crate::*;

#[near_bindgen]
impl Contract {
    #[handle_result]
    #[payable]
    pub fn dao_change_owner(&mut self, new_dao_id: AccountId) -> Result<(), ContractError> {
        check_deposit(ONE_YOCTO)?;
        self.dao.check_owner()?;

        self.dao.dao_id = new_dao_id.into();
        Ok(())
    }

    #[handle_result]
    #[payable]
    pub fn dao_mint(&mut self, amount: U128, reason: String) -> Result<(), ContractError> {
        check_deposit(ONE_YOCTO)?;
        self.dao.check_owner()?;

        Ok(self.mint(amount.into(), Some(&reason)))
    }

    #[handle_result]
    #[payable]
    pub fn dao_change_roke_token_account_id(
        &mut self,
        new_roke_token_account_id: AccountId,
    ) -> Result<(), ContractError> {
        check_deposit(ONE_YOCTO)?;
        self.dao.check_owner()?;

        self.dao.roke_token_account_id = new_roke_token_account_id.into();
        Ok(())
    }

    #[handle_result]
    #[payable]
    pub fn dao_add_oracle(&mut self, new_oracle_id: AccountId) -> Result<(), ContractError> {
        check_deposit(ONE_YOCTO)?;
        self.dao.check_owner()?;

        self.dao.oracles.insert(new_oracle_id, None);
        Ok(())
    }

    #[handle_result]
    #[payable]
    pub fn dao_remove_oracle(&mut self, oracle_id: AccountId) -> Result<(), ContractError> {
        check_deposit(ONE_YOCTO)?;
        self.dao.check_owner()?;

        self.dao.oracles.remove(&oracle_id);
        Ok(())
    }
}
