use crate::*;

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Stream {
    pub owner_id: AccountId,
    pub receiver_id: AccountId,
    pub token_id: TokenId,
    pub balance: Balance,
    pub timestamp_started: Timestamp,
    pub tokens_per_tick: Balance,
    pub tokens_transferred: Balance,
    // TODO refactor StreamStatus
    pub status: StreamStatus,
    // TODO add history of operations
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct StreamView {
    pub owner_id: String,
    pub receiver_id: String,
    pub token_name: String,
    pub balance: WrappedBalance,
    pub timestamp_started: WrappedTimestamp,
    pub tokens_per_tick: WrappedBalance,
    pub tokens_transferred: WrappedBalance,
    pub status: String,
}

impl From<&Stream> for StreamView {
    fn from(s: &Stream) -> Self {
        Self {
            owner_id: s.owner_id.clone(),
            receiver_id: s.receiver_id.clone(),
            token_name: Xyiming::get_token_name_by_id(s.token_id),
            balance: s.balance.into(),
            timestamp_started: s.timestamp_started.into(),
            tokens_per_tick: s.tokens_per_tick.into(),
            tokens_transferred: s.tokens_transferred.into(),
            status: s.status.clone(),
        }
    }
}

impl Xyiming {
    pub(crate) fn extract_stream_or_panic(&mut self, stream_id: &StreamId) -> Stream {
        Self::streams()
            .remove(&stream_id)
            .expect(ERR_STREAM_NOT_EXIST)
    }
}
