use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Token {
    pub account_id: AccountId,

    pub is_listed: bool,

    // taken in current fts in case of listed token
    #[serde(with = "u128_dec_format")]
    pub commission_on_create: Balance,

    // percentage of tokens taken for commission
    pub commission_coef: SafeFloat,

    #[serde(with = "u128_dec_format")]
    pub collected_commission: Balance,

    #[serde(with = "u128_dec_format")]
    pub storage_balance_needed: Balance,

    pub gas_for_ft_transfer: Gas,
    pub gas_for_storage_deposit: Gas,
}

impl Token {
    pub(crate) fn new_unlisted(token_account_id: &AccountId) -> Self {
        Self {
            account_id: token_account_id.clone(),
            is_listed: false,
            commission_on_create: 0, // we don't accept unlisted tokens
            commission_coef: SafeFloat::ZERO,
            collected_commission: 0,
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

impl Contract {
    pub(crate) fn ft_transfer_from_finance(
        &self,
        token_account_id: AccountId,
        receiver: AccountId,
        amount: Balance,
    ) -> Result<Promise, ContractError> {
        if amount == 0 {
            // NEP-141 forbids zero token transfers
            //
            // Return empty promise
            return Ok(Promise::new(receiver));
        }

        // TODO check gas
        Ok(ext_finance_contract::streaming_ft_transfer(
            token_account_id,
            receiver,
            amount.into(),
            self.finance_id.clone(),
            ONE_YOCTO,
            Gas::ONE_TERA * 50,
        ))
    }

    pub(crate) fn ft_transfer_from_self(
        &self,
        token_account_id: AccountId,
        receiver_id: AccountId,
        amount: Balance,
    ) -> Result<Promise, ContractError> {
        if amount == 0 {
            // NEP-141 forbids zero token transfers
            //
            // Return empty promise
            return Ok(Promise::new(receiver_id));
        }

        if Contract::is_aurora_address(&receiver_id) {
            if env::prepaid_gas() - env::used_gas() < MIN_GAS_FOR_AURORA_TRANFSER {
                return Err(ContractError::InsufficientGas {
                    expected: MIN_GAS_FOR_AURORA_TRANFSER,
                    left: env::prepaid_gas() - env::used_gas(),
                });
            }
            if token_account_id == Contract::aurora_account_id() {
                return Ok(ext_fungible_token::ft_transfer_call(
                    Contract::aurora_account_id(),
                    U128(amount),
                    None,
                    Contract::aurora_transfer_call_msg(&receiver_id),
                    Contract::aurora_account_id(),
                    ONE_YOCTO,
                    MIN_GAS_FOR_AURORA_TRANFSER,
                ));
            } else {
                return Ok(ext_fungible_token::ft_transfer_call(
                    Contract::aurora_account_id(),
                    U128(amount),
                    None,
                    receiver_id.to_string(),
                    token_account_id,
                    ONE_YOCTO,
                    MIN_GAS_FOR_AURORA_TRANFSER,
                ));
            }
        } else {
            if env::prepaid_gas() - env::used_gas() < MIN_GAS_FOR_FT_TRANFSER {
                return Err(ContractError::InsufficientGas {
                    expected: MIN_GAS_FOR_FT_TRANFSER,
                    left: env::prepaid_gas() - env::used_gas(),
                });
            }
            Ok(ext_fungible_token::ft_transfer(
                receiver_id,
                U128(amount),
                // TODO write full explanation
                None,
                token_account_id,
                ONE_YOCTO,
                MIN_GAS_FOR_FT_TRANFSER,
            ))
        }
    }
}