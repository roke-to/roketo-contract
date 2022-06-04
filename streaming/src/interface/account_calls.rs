use crate::*;

#[near_bindgen]
impl Contract {
    #[handle_result]
    #[payable]
    pub fn account_update_cron_flag(&mut self, is_cron_allowed: bool) -> Result<(), ContractError> {
        check_deposit(ONE_YOCTO)?;
        // Account actions may be delegated to 3rd parties.
        // That's why it uses env::predecessor_account_id() here and below.
        let mut account = self.extract_account(&env::predecessor_account_id())?;
        account.is_cron_allowed = is_cron_allowed;
        self.save_account(account)
    }

    #[handle_result]
    #[payable]
    pub fn account_deposit_near(&mut self) -> Result<(), ContractError> {
        self.account_deposit(env::predecessor_account_id(), env::attached_deposit())?;
        self.stats_inc_account_deposit(env::attached_deposit(), false);
        Ok(())
    }

    #[handle_result]
    #[payable]
    pub fn account_unstake(&mut self, amount: U128) -> Result<Option<Promise>, ContractError> {
        check_deposit(ONE_YOCTO)?;
        let amount = amount.into();
        let mut account = self.extract_account(&env::predecessor_account_id())?;
        if amount > account.stake {
            return Err(ContractError::InvalidTokenWithdrawAmount {
                requested: amount,
                left: account.stake,
            });
        }

        account.stake -= amount;
        self.save_account(account)?;

        self.ft_transfer_from_self(
            self.dao.get_token(&self.dao.utility_token_id).account_id,
            env::predecessor_account_id(),
            amount,
        )
    }
}

impl Contract {
    pub(crate) fn account_deposit(
        &mut self,
        account_id: AccountId,
        deposit: Balance,
    ) -> Result<(), ContractError> {
        self.create_account_if_not_exist(&account_id)?;
        let mut account = self.extract_account(&account_id)?;
        // this is strongly needed to avoid creating accounts for free
        if account.deposit + deposit < self.dao.commission_non_payment_ft {
            return Err(ContractError::InsufficientDeposit {
                expected: self.dao.commission_non_payment_ft - account.deposit,
                received: deposit,
            });
        }
        account.deposit += deposit;
        self.save_account(account)
    }
}
