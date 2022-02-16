use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Stream {
    #[serde(with = "b58_dec_format")]
    pub id: CryptoHash,
    pub description: Option<String>,
    pub creator_id: AccountId,
    pub owner_id: AccountId,
    pub receiver_id: AccountId,
    pub token_account_id: AccountId,

    pub timestamp_created: Timestamp,
    pub last_action: Timestamp,

    #[serde(with = "u128_dec_format")]
    pub balance: Balance,
    #[serde(with = "u128_dec_format")]
    pub tokens_per_sec: Balance,

    pub status: StreamStatus,
    #[serde(with = "u128_dec_format")]
    pub tokens_total_withdrawn: Balance,

    pub cliff: Option<Timestamp>,

    #[serde(with = "u128_dec_format")]
    pub amount_to_push: Balance,

    pub is_expirable: bool,
    pub is_locked: bool,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VStream {
    Current(Stream),
}

impl From<VStream> for Stream {
    fn from(v: VStream) -> Self {
        match v {
            VStream::Current(c) => c,
        }
    }
}

impl From<Stream> for VStream {
    fn from(c: Stream) -> Self {
        VStream::Current(c)
    }
}

impl Stream {
    pub(crate) fn new(
        description: Option<String>,
        creator_id: AccountId,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_account_id: AccountId,
        balance: Balance,
        tokens_per_sec: Balance,
        cliff: Option<Timestamp>,
        initial_balance: Balance,
        is_expirable: bool,
        is_locked: bool,
    ) -> Stream {
        let id = env::sha256(&env::random_seed())
            .as_slice()
            .try_into()
            .unwrap();
        Self {
            id,
            description,
            creator_id,
            owner_id,
            receiver_id,
            token_account_id,
            timestamp_created: env::block_timestamp(),
            last_action: env::block_timestamp(),
            balance,
            tokens_per_sec,
            status: StreamStatus::Initialized,
            tokens_total_withdrawn: 0,
            cliff,
            amount_to_push: initial_balance,
            is_expirable,
            is_locked,
        }
    }

    pub(crate) fn process_withdraw(&mut self, token: &Token) -> (Balance, Balance) {
        let mut gross_payment = self.available_to_withdraw();
        self.tokens_total_withdrawn += gross_payment;
        let (mut payment, mut commission) = if token.is_listed {
            token.apply_commission(min(gross_payment, self.balance))
        } else {
            (gross_payment, 0)
        };
        if self.cliff.is_some() {
            payment = 0;
            gross_payment = commission;
        }
        if self.is_locked {
            // We already taken the commission while created
            commission = 0;
        }
        if self.balance > gross_payment {
            self.balance -= gross_payment;
        } else {
            self.balance = 0;
            self.status = StreamStatus::Finished {
                reason: StreamFinishReason::FinishedNatually,
            };
        }
        // This update of last_action is useless here
        // however it helps to keep invariant of stream status.
        self.last_action = env::block_timestamp();

        (payment, commission)
    }

    pub(crate) fn available_to_withdraw(&self) -> Balance {
        if self.status == StreamStatus::Active {
            let period = env::block_timestamp() - self.last_action;
            min(
                self.balance,
                (period / TICKS_PER_SECOND) as u128 * self.tokens_per_sec,
            )
        } else {
            0
        }
    }

    pub(crate) fn update_cliff(&mut self) {
        if let Some(cliff) = self.cliff {
            if env::block_timestamp() >= cliff {
                self.cliff = None;
            }
        }
    }
}

impl Contract {
    pub(crate) fn process_action(
        &mut self,
        stream: &mut Stream,
        action_type: ActionType,
    ) -> Result<Vec<Promise>, ContractError> {
        let mut owner = self.extract_account(&stream.owner_id)?;
        let mut receiver = self.extract_account(&stream.receiver_id)?;
        let mut promises = vec![];

        if action_type == ActionType::Init {
            assert!(owner.inactive_outgoing_streams.insert(&stream.id));
            assert!(receiver.inactive_incoming_streams.insert(&stream.id));
        } else {
            assert!(!stream.status.is_terminated());
            match action_type {
                ActionType::Start => {
                    assert!(owner.inactive_outgoing_streams.remove(&stream.id));
                    assert!(receiver.inactive_incoming_streams.remove(&stream.id));
                    assert!(owner.active_outgoing_streams.insert(&stream.id));
                    assert!(receiver.active_incoming_streams.insert(&stream.id));
                    owner
                        .total_outgoing
                        .entry(stream.token_account_id.clone())
                        .and_modify(|e| *e += stream.tokens_per_sec)
                        .or_insert(stream.tokens_per_sec);
                    receiver
                        .total_incoming
                        .entry(stream.token_account_id.clone())
                        .and_modify(|e| *e += stream.tokens_per_sec)
                        .or_insert(stream.tokens_per_sec);
                    stream.status = StreamStatus::Active;
                    self.stats_inc_active_streams(&stream.token_account_id);
                }
                ActionType::Pause => {
                    assert_eq!(stream.status, StreamStatus::Active);
                    promises.push(self.process_payment(stream, &mut receiver)?);
                    assert!(owner.active_outgoing_streams.remove(&stream.id));
                    assert!(receiver.active_incoming_streams.remove(&stream.id));
                    assert!(owner.inactive_outgoing_streams.insert(&stream.id));
                    assert!(receiver.inactive_incoming_streams.insert(&stream.id));
                    owner
                        .total_outgoing
                        .entry(stream.token_account_id.clone())
                        .and_modify(|e| *e -= stream.tokens_per_sec);
                    receiver
                        .total_incoming
                        .entry(stream.token_account_id.clone())
                        .and_modify(|e| *e -= stream.tokens_per_sec);
                    if stream.status == StreamStatus::Active {
                        // The stream may be stopped while payment processing
                        stream.status = StreamStatus::Paused;
                    }
                    self.stats_dec_active_streams(&stream.token_account_id);
                }
                ActionType::Stop { reason } => {
                    if stream.status == StreamStatus::Active {
                        promises.push(self.process_payment(stream, &mut receiver)?);
                        assert!(owner.active_outgoing_streams.remove(&stream.id));
                        assert!(receiver.active_incoming_streams.remove(&stream.id));
                        owner
                            .total_outgoing
                            .entry(stream.token_account_id.clone())
                            .and_modify(|e| *e -= stream.tokens_per_sec);
                        receiver
                            .total_incoming
                            .entry(stream.token_account_id.clone())
                            .and_modify(|e| *e -= stream.tokens_per_sec);
                        self.stats_dec_active_streams(&stream.token_account_id);
                    } else {
                        assert!(owner.inactive_outgoing_streams.remove(&stream.id));
                        assert!(receiver.inactive_incoming_streams.remove(&stream.id));
                    }
                    if !stream.status.is_terminated() {
                        // Refund can be requested only if stream is not terminated naturally yet
                        promises.push(self.process_refund(stream)?);
                        stream.status = StreamStatus::Finished { reason };
                    }
                }
                ActionType::Init => {
                    // Processed separately
                    unreachable!();
                }
                ActionType::Withdraw => {
                    assert_eq!(stream.status, StreamStatus::Active);
                    promises.push(self.process_payment(stream, &mut receiver)?);
                    if stream.status.is_terminated() {
                        assert_eq!(
                            stream.status,
                            StreamStatus::Finished {
                                reason: StreamFinishReason::FinishedNatually
                            }
                        );
                        assert!(owner.active_outgoing_streams.remove(&stream.id));
                        assert!(receiver.active_incoming_streams.remove(&stream.id));
                        owner
                            .total_outgoing
                            .entry(stream.token_account_id.clone())
                            .and_modify(|e| *e -= stream.tokens_per_sec);
                        receiver
                            .total_incoming
                            .entry(stream.token_account_id.clone())
                            .and_modify(|e| *e -= stream.tokens_per_sec);
                        self.stats_dec_active_streams(&stream.token_account_id);
                    }
                }
            }
        }

        stream.last_action = env::block_timestamp();
        self.save_account(owner)?;
        self.save_account(receiver)?;

        Ok(promises)
    }

    fn process_payment(
        &mut self,
        stream: &mut Stream,
        account: &mut Account,
    ) -> Result<Promise, ContractError> {
        let token = self.dao.get_token_or_unlisted(&stream.token_account_id);
        let (payment, commission) = stream.process_withdraw(&token);
        account
            .total_received
            .entry(stream.token_account_id.clone())
            .and_modify(|e| *e += payment)
            .or_insert(payment);
        self.dao
            .tokens
            .entry(stream.token_account_id.clone())
            .and_modify(|e| e.collected_commission += commission);
        self.stats_withdraw(&token, payment, commission);
        self.ft_transfer_from_finance(token.account_id, stream.receiver_id.clone(), payment)
    }

    fn process_refund(&mut self, stream: &mut Stream) -> Result<Promise, ContractError> {
        let token = self.dao.get_token_or_unlisted(&stream.token_account_id);
        let refund = stream.balance;
        stream.balance = 0;
        self.stats_refund(&token, refund);
        self.ft_transfer_from_finance(token.account_id, stream.owner_id.clone(), refund)
    }

    pub(crate) fn view_stream(&mut self, stream_id: &StreamId) -> Result<Stream, ContractError> {
        match self.streams.get(stream_id) {
            Some(vstream) => Ok(vstream.into()),
            None => Err(ContractError::UnreachableStream {
                stream_id: *stream_id,
            }),
        }
    }

    pub(crate) fn extract_stream(&mut self, stream_id: &StreamId) -> Result<Stream, ContractError> {
        match self.streams.remove(stream_id) {
            Some(vstream) => Ok(vstream.into()),
            None => Err(ContractError::UnreachableStream {
                stream_id: *stream_id,
            }),
        }
    }

    pub(crate) fn save_stream(&mut self, stream: Stream) -> Result<(), ContractError> {
        match self.streams.insert(&stream.id.clone(), &stream.into()) {
            None => Ok(()),
            Some(_) => Err(ContractError::DataCorruption),
        }
    }
}