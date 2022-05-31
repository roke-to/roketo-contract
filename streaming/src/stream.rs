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

    // Cliff is a moment of time which divides the stream into two parts:
    // - before the cliff - withdraw is disabled;
    // - after the cliff - stream becomes a regular one.
    //
    // Streams with cliff must be started immediately.
    // Additionally, pausing streams with cliff is disabled
    // because it's hard to predict the proper behavior of the stream.
    //
    // The only way to withdraw tokens from stream with active cliff
    // is to stop it completely. In this case we take commission
    // that is proportional to time passed for total stream time.
    //
    // The reason of having cliffs is to reproduce vesting contracts.
    pub cliff: Option<Timestamp>,

    // Stream non-expiration is a hard concept to understand.
    //
    // The idea is based on observation that no stream can be stopped
    // automatically with no action provided. So, if receiver haven't
    // withdraw his tokens from fully expired stream yet,
    // the stream is considered Active.
    //
    // This basically means, the owner can deposit tokens onto the stream
    // even it's already expired, as long as receiver haven't tried to withdraw
    // the tokens that leads to stream finishing. In other terms,
    // it's possible to have a credit that may be covered later.
    //
    // Such behavior called non-expirable streams and disabled by default.
    // Expirable streams will be terminated even on stream depositing.
    pub is_expirable: bool,

    // Locked streams are ones that are unable to pause, stop and change receiver.
    // Locked streams are still may be terminated or deposited until started.
    //
    // For locked streams we take commission when the stream is started,
    // to allow us to own and handle commission tokens without waiting
    // as the final result of locked stream cannot be changed.
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
        salt: u64,
        description: Option<String>,
        creator_id: AccountId,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_account_id: AccountId,
        balance: Balance,
        tokens_per_sec: Balance,
        cliff: Option<Timestamp>,
        is_expirable: bool,
        is_locked: bool,
    ) -> Stream {
        let mut buf = env::random_seed();
        buf.append(&mut salt.to_le_bytes().to_vec());
        let id = env::sha256(&buf).as_slice().try_into().unwrap();
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
            is_expirable,
            is_locked,
        }
    }

    pub(crate) fn process_withdraw(&mut self, token: &Token) -> (Balance, Balance) {
        let mut gross_payment = self.available_to_withdraw();
        assert!(
            gross_payment <= self.balance,
            "available_to_withdraw() must guarantee that gross_payment({}) <= self.balance({})",
            gross_payment,
            self.balance
        );
        let (mut payment, mut commission) = if token.is_payment {
            token.apply_commission(gross_payment)
        } else {
            (gross_payment, 0)
        };
        if self.cliff.is_some() {
            payment = 0;
            gross_payment = commission;
        }
        self.tokens_total_withdrawn += gross_payment;
        if self.is_locked {
            // We already taken the commission while created
            commission = 0;
        }

        self.balance -= gross_payment;

        if self.balance == 0 {
            self.status = StreamStatus::Finished {
                reason: StreamFinishReason::FinishedNaturally,
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
        check_integrity(!stream.status.is_terminated())?;

        let mut owner = self.extract_account(&stream.owner_id)?;
        let mut receiver = self.extract_account(&stream.receiver_id)?;
        let mut promises = vec![];

        if action_type == ActionType::Init {
            check_integrity(owner.inactive_outgoing_streams.insert(&stream.id))?;
            check_integrity(receiver.inactive_incoming_streams.insert(&stream.id))?;
        } else {
            // No action is applicable for terminated stream.
            match action_type {
                ActionType::Start => {
                    check_integrity(owner.inactive_outgoing_streams.remove(&stream.id))?;
                    check_integrity(receiver.inactive_incoming_streams.remove(&stream.id))?;
                    check_integrity(owner.active_outgoing_streams.insert(&stream.id))?;
                    check_integrity(receiver.active_incoming_streams.insert(&stream.id))?;
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
                    check_integrity(stream.status == StreamStatus::Active)?;
                    if let Some(promise) = self.process_payment(stream, &mut receiver)? {
                        promises.push(promise);
                    }
                    owner
                        .total_outgoing
                        .entry(stream.token_account_id.clone())
                        .and_modify(|e| *e -= stream.tokens_per_sec);
                    receiver
                        .total_incoming
                        .entry(stream.token_account_id.clone())
                        .and_modify(|e| *e -= stream.tokens_per_sec);
                    check_integrity(owner.active_outgoing_streams.remove(&stream.id))?;
                    check_integrity(receiver.active_incoming_streams.remove(&stream.id))?;
                    if !stream.status.is_terminated() {
                        check_integrity(owner.inactive_outgoing_streams.insert(&stream.id))?;
                        check_integrity(receiver.inactive_incoming_streams.insert(&stream.id))?;
                    }
                    if stream.status == StreamStatus::Active {
                        // The stream may be stopped while payment processing
                        stream.status = StreamStatus::Paused;
                    }
                    self.stats_dec_active_streams(&stream.token_account_id);
                }
                ActionType::Stop { reason } => {
                    if stream.status == StreamStatus::Active {
                        if let Some(promise) = self.process_payment(stream, &mut receiver)? {
                            promises.push(promise);
                        }
                        check_integrity(owner.active_outgoing_streams.remove(&stream.id))?;
                        check_integrity(receiver.active_incoming_streams.remove(&stream.id))?;
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
                        check_integrity(owner.inactive_outgoing_streams.remove(&stream.id))?;
                        check_integrity(receiver.inactive_incoming_streams.remove(&stream.id))?;
                    }
                    if !stream.status.is_terminated() {
                        // Refund can be requested only if stream is not terminated naturally yet
                        if let Some(promise) = self.process_refund(stream)? {
                            promises.push(promise);
                        }
                        stream.status = StreamStatus::Finished { reason };
                    }
                }
                ActionType::Init => {
                    // Processed separately
                    unreachable!();
                }
                ActionType::Withdraw => {
                    check_integrity(stream.status == StreamStatus::Active)?;
                    if let Some(promise) = self.process_payment(stream, &mut receiver)? {
                        promises.push(promise);
                    }
                    if stream.status.is_terminated() {
                        check_integrity(
                            stream.status
                                == StreamStatus::Finished {
                                    reason: StreamFinishReason::FinishedNaturally,
                                },
                        )?;
                        check_integrity(owner.active_outgoing_streams.remove(&stream.id))?;
                        check_integrity(receiver.active_incoming_streams.remove(&stream.id))?;
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
    ) -> Result<Option<Promise>, ContractError> {
        let token = self.dao.get_token(&stream.token_account_id);
        let (payment, commission) = stream.process_withdraw(&token);
        account
            .total_received
            .entry(stream.token_account_id.clone())
            .and_modify(|e| *e += payment)
            .or_insert(payment);
        self.stats_withdraw(&token, payment, commission);
        self.ft_transfer_from_finance(token.account_id, stream.receiver_id.clone(), payment)
    }

    fn process_refund(&mut self, stream: &mut Stream) -> Result<Option<Promise>, ContractError> {
        let token = self.dao.get_token(&stream.token_account_id);
        let refund = stream.balance;
        stream.balance = 0;
        self.stats_refund(&token, refund);
        self.ft_transfer_from_finance(token.account_id, stream.owner_id.clone(), refund)
    }

    pub(crate) fn view_stream(&self, stream_id: &StreamId) -> Result<Stream, ContractError> {
        match self.streams.get(stream_id) {
            Some(vstream) => Ok(vstream.into()),
            None => Err(ContractError::StreamNotExist {
                stream_id: *stream_id,
            }),
        }
    }

    pub(crate) fn extract_stream(&mut self, stream_id: &StreamId) -> Result<Stream, ContractError> {
        match self.streams.remove(stream_id) {
            Some(vstream) => Ok(vstream.into()),
            None => Err(ContractError::StreamNotExist {
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
