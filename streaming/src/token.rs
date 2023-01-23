use near_contract_standards::fungible_token::core::ext_ft_core;

use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Token {
    pub account_id: AccountId,

    pub is_payment: bool,

    // taken in current fts in case of listed token
    #[serde(with = "u128_dec_format")]
    pub commission_on_create: Balance,

    // percentage of tokens taken for commission
    pub commission_coef: SafeFloat,

    // taken in current fts in case of listed token
    #[serde(with = "u128_dec_format")]
    pub commission_on_transfer: Balance,

    #[serde(with = "u128_dec_format")]
    pub storage_balance_needed: Balance,
    pub gas_for_ft_transfer: Gas,
    pub gas_for_storage_deposit: Gas,
}

impl Token {
    pub(crate) fn new_unlisted(token_account_id: &AccountId) -> Self {
        Self {
            account_id: token_account_id.clone(),
            is_payment: false,
            commission_on_create: 0, // we don't accept unlisted tokens
            commission_coef: SafeFloat::ZERO,
            commission_on_transfer: 0, // we don't accept unlisted tokens
            storage_balance_needed: DEFAULT_STORAGE_BALANCE,
            gas_for_ft_transfer: DEFAULT_GAS_FOR_FT_TRANSFER,
            gas_for_storage_deposit: DEFAULT_GAS_FOR_STORAGE_DEPOSIT,
        }
    }

    pub(crate) fn apply_commission(&self, amount: Balance) -> (Balance, Balance) {
        // round commission up
        if self.commission_coef == SafeFloat::ZERO || amount == 0 {
            (amount, 0)
        } else {
            let commission = self.commission_coef.mult_safe(amount - 1) + 1;
            (amount - commission, commission)
        }
    }
}

#[ext_contract]
pub trait ExtFinanceContract {
    fn streaming_ft_transfer(
        &mut self,
        token_account_id: AccountId,
        receiver: AccountId,
        amount: U128,
    ) -> Promise;
}

#[ext_contract]
pub trait ExtStorageManagement {
    fn storage_deposit(&mut self, account_id: Option<AccountId>, registration_only: Option<bool>);
}

impl Contract {
    pub(crate) fn ft_transfer_from_finance(
        &self,
        token_account_id: AccountId,
        receiver: AccountId,
        amount: Balance,
    ) -> Result<Option<Promise>, ContractError> {
        if amount == 0 {
            // NEP-141 forbids zero token transfers
            return Ok(None);
        }

        // TODO #16
        let gas_needed = Gas::ONE_TERA * 50;
        check_gas(gas_needed)?;
        let promise = ext_finance_contract::ext(self.finance_id.clone())
            .with_attached_deposit(ONE_YOCTO)
            .with_static_gas(gas_needed)
            .streaming_ft_transfer(token_account_id, receiver, U128(amount));
        Ok(Some(promise))
    }

    pub(crate) fn ft_transfer_from_self(
        &self,
        token_account_id: AccountId,
        receiver_id: AccountId,
        amount: Balance,
    ) -> Result<Option<Promise>, ContractError> {
        if amount == 0 {
            // NEP-141 forbids zero token transfers
            return Ok(None);
        }

        if is_aurora_address(&receiver_id) {
            check_gas(MIN_GAS_FOR_AURORA_TRANFSER)?;
            if token_account_id == aurora_account_id() {
                let promise = ext_ft_core::ext(aurora_account_id())
                    .with_attached_deposit(ONE_YOCTO)
                    .with_static_gas(MIN_GAS_FOR_AURORA_TRANFSER)
                    .ft_transfer_call(
                        aurora_account_id(),
                        U128(amount),
                        None,
                        aurora_transfer_call_msg(&receiver_id),
                    );
                Ok(Some(promise))
            } else {
                let promise = ext_ft_core::ext(token_account_id)
                    .with_attached_deposit(ONE_YOCTO)
                    .with_static_gas(MIN_GAS_FOR_AURORA_TRANFSER)
                    .ft_transfer_call(
                        aurora_account_id(),
                        U128(amount),
                        None,
                        receiver_id.to_string(),
                    );
                Ok(Some(promise))
            }
        } else {
            check_gas(MIN_GAS_FOR_FT_TRANFSER)?;
            let promise = ext_ft_core::ext(token_account_id)
                .with_attached_deposit(ONE_YOCTO)
                .with_static_gas(MIN_GAS_FOR_FT_TRANFSER)
                .ft_transfer(receiver_id, U128(amount), None); // TODO write full explanation
            Ok(Some(promise))
        }
    }
}
