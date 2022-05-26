use crate::*;

#[ext_contract(ext_self)]
pub trait ExtTransferUnwrapped {
    fn on_near_unwrapped(&mut self, account_id: AccountId, amount: U128) -> Promise;
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn on_near_unwrapped(&mut self, account_id: AccountId, amount: U128) -> Promise {
        Promise::new(account_id).transfer(amount.into())
    }
}

impl Contract {
    pub(crate) fn ft_transfer(
        &self,
        token_account_id: AccountId,
        receiver: AccountId,
        amount: Balance,
    ) -> Promise {
        if amount == 0 {
            // NEP-141 forbids zero token transfers
            //
            // Return empty promise
            return Promise::new(receiver);
        }

        if is_aurora_address(&receiver) {
            if token_account_id == aurora_account_id() {
                return ext_fungible_token::ft_transfer_call(
                    aurora_account_id(),
                    U128(amount),
                    None,
                    aurora_transfer_call_msg(&receiver),
                    aurora_account_id(),
                    ONE_YOCTO,
                    env::prepaid_gas() - env::used_gas() - Gas::ONE_TERA * 10,
                );
            } else {
                return ext_fungible_token::ft_transfer_call(
                    aurora_account_id(),
                    U128(amount),
                    None,
                    receiver.to_string(),
                    token_account_id,
                    ONE_YOCTO,
                    env::prepaid_gas() - env::used_gas() - Gas::ONE_TERA * 10,
                );
            }
        } else if token_account_id == wrap_near_account_id() {
            ext_wrap_near::near_withdraw(
                U128(amount),
                wrap_near_account_id(),
                ONE_YOCTO,
                Gas::ONE_TERA * 10,
            )
            .then(ext_self::on_near_unwrapped(
                receiver,
                U128(amount),
                env::current_account_id(),
                0, // no deposit
                Gas::ONE_TERA * 10,
            ))
        } else {
            ext_fungible_token::ft_transfer(
                receiver,
                U128(amount),
                None,
                token_account_id,
                ONE_YOCTO,
                env::prepaid_gas() - env::used_gas() - Gas::ONE_TERA * 10,
            )
        }
    }
}
