use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Dao {
    pub dao_id: AccountId,

    #[serde(with = "u128_dec_format")]
    pub unlock_price: Balance,

    pub unlock_period_sec: u64,

    pub roke_token_account_id: AccountId,

    pub oracles: HashMap<AccountId, Option<Timestamp>>,
}

impl Dao {
    pub(crate) fn new(dao_id: AccountId, unlock_price: Balance) -> Self {
        Self {
            dao_id,
            unlock_price,
            unlock_period_sec: DEFAULT_ROKE_DERIVATIVE_UNLOCK_PERIOD,
            roke_token_account_id: AccountId::new_unchecked(ROKE_TOKEN_ACCOUNT_STR.to_string()),
            oracles: HashMap::new(),
        }
    }

    pub(crate) fn check_owner(&self) -> Result<(), ContractError> {
        // The call must be executed from dao account.
        if env::predecessor_account_id() == self.dao_id {
            Ok(())
        } else {
            Err(ContractError::CallerIsNotDao {
                expected: self.dao_id.clone(),
                received: env::predecessor_account_id(),
            })
        }
    }

    pub(crate) fn check_oracle(&self, sender_id: &AccountId) -> Result<(), ContractError> {
        match self.oracles.get(sender_id) {
            Some(_) => Ok(()),
            None => Err(ContractError::UnknownOracle {
                received: sender_id.clone(),
            }),
        }
    }
}
