use crate::*;

#[near_bindgen]
impl Contract {
    #[handle_result]
    #[payable]
    pub fn oracle_update_roke_usn(&mut self, price: U128) -> Result<(), ContractError> {
        // Oracle actions may be delegated to 3rd parties.
        // That's why it uses env::predecessor_account_id() here and below.
        self.dao.check_oracle(&env::predecessor_account_id())?;
        let price: Balance = price.into();
        self.dao
            .oracles
            .entry(env::predecessor_account_id())
            .and_modify(|e| {
                *e = if price >= self.dao.unlock_price {
                    if e.is_some() {
                        *e
                    } else {
                        Some(env::block_timestamp())
                    }
                } else {
                    None
                }
            });
        Ok(())
    }
}
