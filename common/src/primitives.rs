use crate::*;

pub const NO_DEPOSIT: Balance = 0;

pub const MAX_DESCRIPTION_LEN: usize = 255;

pub const MIN_STREAMING_SPEED: u128 = 1;
pub const MAX_STREAMING_SPEED: u128 = 10u128.pow(27 as _); // 1e27

pub const MAX_AMOUNT: u128 = 10u128.pow(33 as _); // 1e33

pub const TICKS_PER_SECOND: u64 = 10u64.pow(9 as _); // 1e9

pub const ONE_TERA: u64 = Gas::ONE_TERA.0; // near-sdk Gas is totally useless

// Commission in NEAR for tokens that we don't want to accept as payment.
pub const DEFAULT_COMMISSION_NON_PAYMENT_FT: Balance = ONE_NEAR / 10; // 0.1 NEAR

pub const DEFAULT_VIEW_STREAMS_LIMIT: u32 = 10;
pub const STORAGE_NEEDS_PER_STREAM: Balance = ONE_NEAR / 20; // 0.05 NEAR

// Explanation on default storage balance and gas needs.
//
// Normally it's enough to take 0.00125 NEAR for storage deposit
// and ~10 TGas for transfers and storage deposit
// for most regular fungible tokens based on NEP-141 standard.
// However, custom tokens may reqiure high amounts of NEAR
// for storage uses and needs more gas for complex calculations
// happens within transfers.
// To allow those custom tokens be transferable by the contract,
// the default limits were increased deliberately.
pub const DEFAULT_STORAGE_BALANCE: Balance = ONE_NEAR / 10;
pub const DEFAULT_GAS_FOR_FT_TRANSFER: Gas = Gas(50 * ONE_TERA);
pub const DEFAULT_GAS_FOR_STORAGE_DEPOSIT: Gas = Gas(25 * ONE_TERA);
// In cases to avoid high storage deposit and gas needs,
// or if the defaults are not enough for you token,
// ask DAO to whitelist the token with proper values.

pub const MIN_GAS_FOR_AURORA_TRANFSER: Gas = Gas(70 * ONE_TERA);
pub const MIN_GAS_FOR_FT_TRANFSER: Gas = Gas(20 * ONE_TERA);

pub const ROKE_TOKEN_DECIMALS: u8 = 18;
pub const ROKE_TOKEN_TOTAL_SUPPLY: u128 = 100_000_000 * 10u128.pow(ROKE_TOKEN_DECIMALS as _);
pub const ROKE_TOKEN_SYMBOL: &str = "ROKE";
pub const ROKE_TOKEN_NAME: &str = "Roke.to Streaming Token";
pub const ROKE_TOKEN_SVG_ICON: &str = "data:image/jpeg;base64,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";

#[ext_contract(ext_finance)]
pub trait ExtFinance {
    fn streaming_storage_needs_transfer(&mut self) -> Promise;
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SafeFloat {
    pub val: u32,
    pub pow: i8,
}

impl SafeFloat {
    pub const MAX_SAFE: u128 = 10u128.pow(27 as _); // 1e27

    pub const ZERO: SafeFloat = SafeFloat { val: 0, pow: 0 };

    pub fn assert_safe(&self) {
        self.mult_safe(MAX_AMOUNT);
    }

    pub fn assert_safe_commission(&self) {
        self.assert_safe();
        assert_eq!(self.mult_safe(1), 0);
    }

    pub fn mult_safe(&self, x: u128) -> u128 {
        if self.pow < 0 {
            let mut cur = x;
            let mut p = -self.pow;
            while cur > SafeFloat::MAX_SAFE && p > 0 {
                cur /= 10;
                p -= 1;
            }
            // Hold multiplier not higher than 1e27, so max value here may be
            // 1e27 * 2e32 ==
            // 4294967296000000000000000000000000000
            //
            // while limit of u128 is
            // 2e128 ==
            // 340282366920938463463374607431768211456
            //
            // It keeps final multiplication within boundaries
            cur * self.val as u128 / 10u128.pow(p as _)
        } else {
            // Not too much we can do here, right?
            x * self.val as u128 * 10u128.pow(self.pow as _)
        }
    }
}

pub fn check_deposit(deposit_needed: Balance) -> Result<(), ContractError> {
    if env::attached_deposit() >= deposit_needed {
        Ok(())
    } else {
        Err(ContractError::InsufficientDeposit {
            expected: deposit_needed,
            received: env::attached_deposit(),
        })
    }
}

pub fn check_gas(gas_needed: Gas) -> Result<(), ContractError> {
    if env::prepaid_gas() - env::used_gas() >= gas_needed {
        Ok(())
    } else {
        Err(ContractError::InsufficientGas {
            expected: gas_needed,
            left: env::prepaid_gas() - env::used_gas(),
        })
    }
}

pub fn check_integrity(action_result: bool) -> Result<(), ContractError> {
    if action_result {
        Ok(())
    } else {
        Err(ContractError::DataCorruption)
    }
}

pub type StreamId = CryptoHash;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum StreamStatus {
    Initialized,
    Active,
    Paused,
    Finished { reason: StreamFinishReason },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum StreamFinishReason {
    StoppedByOwner,
    StoppedByReceiver,
    FinishedNaturally,
    FinishedBecauseCannotBeExtended,
    FinishedWhileTransferred,
}

impl StreamStatus {
    pub fn is_terminated(&self) -> bool {
        match self {
            StreamStatus::Initialized => false,
            StreamStatus::Active => false,
            StreamStatus::Paused => false,
            StreamStatus::Finished { reason: _ } => true,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum ActionType {
    Init,
    Start,
    Pause,
    Withdraw,
    Stop { reason: StreamFinishReason },
}

pub mod u128_dec_format {
    use near_sdk::serde::de;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&num.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

pub mod b58_dec_format {
    use near_sdk::json_types::Base58CryptoHash;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};
    use near_sdk::CryptoHash;

    pub fn serialize<S>(val: &CryptoHash, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // impossible to do without intermediate serialization
        serializer.serialize_str(&String::from(&Base58CryptoHash::from(*val)))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<CryptoHash, D::Error>
    where
        D: Deserializer<'de>,
    {
        // same as above
        Ok(Base58CryptoHash::deserialize(deserializer)?.into())
    }
}
