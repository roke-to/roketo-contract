use near_contract_standards::fungible_token::core::ext_ft_core;

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
                let promise = ext_ft_core::ext(aurora_account_id())
                    .with_attached_deposit(ONE_YOCTO)
                    .with_static_gas(env::prepaid_gas() - env::used_gas() - Gas::ONE_TERA * 10)
                    .ft_transfer_call(
                        aurora_account_id(),
                        U128(amount),
                        None,
                        aurora_transfer_call_msg(&receiver),
                    );
                return promise;
            } else {
                let promise = ext_ft_core::ext(token_account_id)
                    .with_attached_deposit(ONE_YOCTO)
                    .with_static_gas(env::prepaid_gas() - env::used_gas() - Gas::ONE_TERA * 10)
                    .ft_transfer_call(
                        aurora_account_id(),
                        U128(amount),
                        None,
                        receiver.to_string(),
                    );
                return promise;
            }
        } else if token_account_id == wrap_near_account_id() {
            let near_withdraw_promise = ext_wrap_near::ext(wrap_near_account_id())
                .with_attached_deposit(ONE_YOCTO)
                .with_static_gas(Gas::ONE_TERA * 10)
                .near_withdraw(U128(amount));
            let on_near_unwrapped_promise = ext_self::ext(env::current_account_id())
                .with_static_gas(Gas::ONE_TERA * 10)
                .on_near_unwrapped(receiver, U128(amount));
            near_withdraw_promise.then(on_near_unwrapped_promise)
        } else {
            ext_ft_core::ext(token_account_id)
                .with_attached_deposit(ONE_YOCTO)
                .with_static_gas(env::prepaid_gas() - env::used_gas() - Gas::ONE_TERA * 10)
                .ft_transfer(receiver, U128(amount), None)
        }
    }

    pub(crate) fn ft_transfer_call(
        &self,
        token_account_id: AccountId,
        receiver: AccountId,
        amount: Balance,
        msg: String,
    ) -> Promise {
        if amount == 0 {
            // NEP-141 forbids zero token transfers
            //
            // Return empty promise
            return Promise::new(receiver);
        }

        if is_aurora_address(&receiver) {
            unimplemented!();
        }
        ext_ft_core::ext(token_account_id)
            .with_attached_deposit(ONE_YOCTO)
            .with_static_gas(env::prepaid_gas() - env::used_gas() - Gas::ONE_TERA * 10)
            .ft_transfer_call(receiver, U128(amount), None, msg)
    }
}
