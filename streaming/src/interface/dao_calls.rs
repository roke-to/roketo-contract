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
    pub fn dao_update_token(&mut self, token: Token) -> Result<(), ContractError> {
        check_deposit(ONE_YOCTO)?;
        self.dao.check_owner()?;

        if self.dao.tokens.remove(&token.account_id).is_none() {
            self.stats_add_token(&token.account_id);
        }

        token.commission_coef.assert_safe_commission();
        self.dao.tokens.insert(token.account_id.clone(), token);
        Ok(())
    }

    #[handle_result]
    #[payable]
    pub fn dao_update_commission_unlisted(
        &mut self,
        commission_unlisted: U128,
    ) -> Result<(), ContractError> {
        check_deposit(ONE_YOCTO)?;
        self.dao.check_owner()?;

        self.dao.commission_unlisted = commission_unlisted.into();
        Ok(())
    }

    #[handle_result]
    #[payable]
    pub fn dao_withdraw_ft(
        &mut self,
        token_account_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> Result<Promise, ContractError> {
        check_deposit(ONE_YOCTO)?;
        self.dao.check_owner()?;

        self.ft_transfer_from_self(token_account_id, receiver_id, amount.into())
    }

    #[handle_result]
    #[payable]
    pub fn dao_withdraw_near(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
    ) -> Result<Promise, ContractError> {
        check_deposit(ONE_YOCTO)?;
        self.dao.check_owner()?;

        Ok(Promise::new(receiver_id).transfer(amount.into()))
    }

    #[handle_result]
    #[payable]
    pub fn dao_add_oracle(&mut self, new_oracle_id: AccountId) -> Result<(), ContractError> {
        check_deposit(ONE_YOCTO)?;
        self.dao.check_owner()?;

        self.dao.oracles.insert(new_oracle_id);
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
