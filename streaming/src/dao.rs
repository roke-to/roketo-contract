use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Dao {
    pub dao_id: AccountId,

    pub tokens: HashMap<AccountId, Token>,
    #[serde(with = "u128_dec_format")]
    pub commission_non_payment_ft: Balance,

    pub utility_token_id: AccountId,
    pub utility_token_decimals: u8,

    // Related to charges in Aurora
    pub eth_near_ratio: SafeFloat,

    pub oracles: HashSet<AccountId>,

    pub approved_nfts: HashSet<AccountId>,
}

impl Dao {
    pub(crate) fn new(
        dao_id: AccountId,
        utility_token_id: AccountId,
        utility_token_decimals: u8,
    ) -> Self {
        Self {
            dao_id,
            tokens: HashMap::new(),
            commission_non_payment_ft: DEFAULT_COMMISSION_NON_PAYMENT_FT,
            utility_token_id,
            utility_token_decimals,
            eth_near_ratio: SafeFloat::ZERO,
            oracles: HashSet::new(),
            approved_nfts: HashSet::new(),
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

    pub(crate) fn get_token(&self, token_account_id: &AccountId) -> Token {
        match self.tokens.get(token_account_id) {
            Some(token) => token.clone(),
            None => Token::new_unlisted(token_account_id),
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
