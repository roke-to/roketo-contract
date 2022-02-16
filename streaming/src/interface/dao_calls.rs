use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn dao_change_owner(&mut self, new_dao_id: AccountId) {
        assert_one_yocto();
        self.dao.check_owner().unwrap();

        self.dao.dao_id = new_dao_id.into();
    }

    #[payable]
    pub fn dao_update_token(&mut self, token: Token) {
        assert_one_yocto();
        self.dao.check_owner().unwrap();

        if self.dao.tokens.remove(&token.account_id).is_none() {
            self.stats_add_token(&token.account_id);
        }

        token.commission_coef.assert_safe_commission();
        self.dao.tokens.insert(token.account_id.clone(), token);
    }

    #[payable]
    pub fn dao_update_commission_unlisted(&mut self, commission_unlisted: U128) {
        assert_one_yocto();
        self.dao.check_owner().unwrap();

        self.dao.commission_unlisted = commission_unlisted.into();
    }

    #[payable]
    pub fn dao_withdraw_ft(
        &mut self,
        token_account_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> Promise {
        assert_one_yocto();
        self.dao.check_owner().unwrap();

        self.ft_transfer_from_self(token_account_id, receiver_id, amount.into())
            .unwrap()
    }

    #[payable]
    pub fn dao_withdraw_near(&mut self, receiver_id: AccountId, amount: U128) -> Promise {
        assert_one_yocto();
        self.dao.check_owner().unwrap();

        Promise::new(receiver_id).transfer(amount.into())
    }

    #[payable]
    pub fn dao_add_oracle(&mut self, new_oracle_id: AccountId) {
        assert_one_yocto();
        self.dao.check_owner().unwrap();

        self.dao.oracles.insert(new_oracle_id);
    }

    #[payable]
    pub fn dao_remove_oracle(&mut self, oracle_id: AccountId) {
        assert_one_yocto();
        self.dao.check_owner().unwrap();

        self.dao.oracles.remove(&oracle_id);
    }
}
