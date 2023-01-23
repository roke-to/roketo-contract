use near_contract_standards::fungible_token::core::ext_ft_core;

use crate::*;

#[near_bindgen]
impl Contract {
    #[handle_result]
    #[payable]
    pub fn unlock(&mut self, amount: U128) -> Result<Promise, ContractError> {
        check_deposit(ONE_YOCTO)?;

        if self.dao.oracles.is_empty() {
            return Err(ContractError::NoOraclesFound);
        }

        // All oralces must approve unlock
        for (oracle, report) in self.dao.oracles.drain() {
            match report {
                Some(timestamp) => {
                    if env::block_timestamp() - timestamp
                        < self.dao.unlock_period_sec * TICKS_PER_SECOND
                    {
                        return Err(ContractError::UnlockPeriodNotPassed { timestamp });
                    }
                }
                None => {
                    return Err(ContractError::OracleNotApproved { oracle });
                }
            }
        }

        let amount = amount.into();
        let left = self
            .token
            .internal_unwrap_balance_of(&env::predecessor_account_id());
        if left <= amount {
            return Err(ContractError::InvalidTokenWithdrawAmount {
                requested: amount,
                left,
            });
        }

        self.burn(amount);

        // TODO process withdraw failure if needed
        let promise = ext_ft_core::ext(self.dao.roke_token_account_id.clone())
            .with_attached_deposit(ONE_YOCTO)
            .with_static_gas(Gas::ONE_TERA * 30) // TODO gas
            .ft_transfer(env::predecessor_account_id(), U128(amount), None);
        Ok(promise)
    }
}
