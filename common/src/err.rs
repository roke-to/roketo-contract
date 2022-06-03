use crate::*;

use near_sdk::FunctionError;

#[derive(BorshDeserialize, BorshSerialize, Serialize, PartialEq, Debug)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize))]
#[serde(crate = "near_sdk::serde")]
pub enum ContractError {
    Unknown,
    CallerIsNotDao {
        expected: AccountId,
        received: AccountId,
    },
    CallerIsNotStreamOwner {
        expected: AccountId,
        received: AccountId,
    },
    CallerIsNotStreamActor {
        owner: AccountId,
        receiver: AccountId,
        caller: AccountId,
    },
    NoOraclesFound,
    UnknownOracle {
        received: AccountId,
    },
    OracleNotApproved {
        oracle: AccountId,
    },
    UnknownToken {
        received: AccountId,
    },
    InvalidToken {
        expected: AccountId,
        received: AccountId,
    },
    ZeroTokenTransfer,
    ZeroBalanceStreamStart,
    CronCallsForbidden {
        received: AccountId,
    },
    MustStartImmediately,
    CannotStartStream {
        stream_status: StreamStatus,
    },
    CannotPauseStream {
        stream_status: StreamStatus,
    },
    CannotStopStream {
        stream_status: StreamStatus,
    },
    CannotWithdraw {
        stream_status: StreamStatus,
    },
    CliffNotPassed {
        timestamp: u64,
    },
    UnlockPeriodNotPassed {
        timestamp: u64,
    },
    InvalidCommission,
    InsufficientGas {
        expected: Gas,
        left: Gas,
    },
    InsufficientDeposit {
        #[serde(with = "u128_dec_format")]
        expected: Balance,
        #[serde(with = "u128_dec_format")]
        received: Balance,
    },
    InsufficientNearBalance {
        #[serde(with = "u128_dec_format")]
        requested: Balance,
        #[serde(with = "u128_dec_format")]
        left: Balance,
    },
    InvalidTokenWithdrawAmount {
        #[serde(with = "u128_dec_format")]
        requested: Balance,
        #[serde(with = "u128_dec_format")]
        left: Balance,
    },
    NFTNotApproved {
        account_id: AccountId,
    },
    AccountNotExist {
        account_id: AccountId,
    },
    StreamNotExist {
        #[serde(with = "b58_dec_format")]
        stream_id: CryptoHash,
    },
    StreamLocked {
        #[serde(with = "b58_dec_format")]
        stream_id: CryptoHash,
    },
    StreamTerminated {
        #[serde(with = "b58_dec_format")]
        stream_id: CryptoHash,
    },
    StreamExpired {
        #[serde(with = "b58_dec_format")]
        stream_id: CryptoHash,
    },
    DescriptionTooLong {
        max_description_len: usize,
        received: usize,
    },
    InvalidStreamingSpeed {
        #[serde(with = "u128_dec_format")]
        min_streaming_speed: u128,
        #[serde(with = "u128_dec_format")]
        max_streaming_speed: u128,
        #[serde(with = "u128_dec_format")]
        received: u128,
    },
    ExceededMaxBalance {
        #[serde(with = "u128_dec_format")]
        max_amount: Balance,
    },
    PredecessorIsNotOwner {
        expected: AccountId,
        received: AccountId,
    },
    DataCorruption,
}

impl FunctionError for ContractError {
    fn panic(&self) -> ! {
        crate::env::panic_str(
            &serde_json::to_string(self).unwrap_or(format!("serde failed: {:?}", self)),
        )
    }
}
