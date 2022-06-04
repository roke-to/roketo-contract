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
        Ok(ext_fungible_token::ft_transfer(
            env::predecessor_account_id(),
            U128(amount),
            None,
            self.dao.roke_token_account_id.clone(),
            ONE_YOCTO,
            Gas::ONE_TERA * 30, // TODO gas
        ))
    }
}
