use crate::*;

impl Contract {
    pub(crate) fn process_create_stream(
        &mut self,
        description: Option<String>,
        creator_id: AccountId,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_account_id: AccountId,
        initial_balance: Balance,
        tokens_per_sec: Balance,
        cliff_period_sec: Option<u32>,
        is_auto_start_enabled: Option<bool>,
        is_expirable: Option<bool>,
        is_locked: Option<bool>,
    ) -> Result<(), ContractError> {
        // NEP-141 forbids zero-token transfers, so this should never happen.
        assert_ne!(initial_balance, 0);

        if description.is_some() && description.clone().unwrap().len() >= MAX_DESCRIPTION_LEN {
            return Err(ContractError::DescriptionTooLong {
                max_description_len: MAX_DESCRIPTION_LEN,
                received: description.clone().unwrap().len(),
            });
        }
        if tokens_per_sec == 0 || tokens_per_sec > MAX_STREAMING_SPEED {
            return Err(ContractError::InvalidStreamingSpeed {
                min_streaming_speed: MIN_STREAMING_SPEED,
                max_streaming_speed: MAX_STREAMING_SPEED,
                received: tokens_per_sec,
            });
        }
        let is_auto_start_enabled = match is_auto_start_enabled {
            Some(value) => value,
            None => true,
        };
        let is_expirable = match is_expirable {
            Some(value) => value,
            None => true,
        };
        let is_locked = match is_locked {
            Some(value) => value,
            None => false,
        };

        self.create_account_if_not_exist(&creator_id)?;
        self.create_account_if_not_exist(&owner_id)?;
        self.create_account_if_not_exist(&receiver_id)?;

        let mut creator = self.extract_account(&creator_id)?;
        let mut balance = initial_balance;

        let mut token = self.dao.get_token_or_unlisted(&token_account_id);
        let is_listed = token.is_listed;
        let mut commission = 0;

        if is_listed {
            // Take commission as DAO proposed
            if balance < token.commission_on_create {
                return Err(ContractError::InsufficientNearDeposit {
                    expected: token.commission_on_create,
                    received: balance,
                });
            }
            balance -= token.commission_on_create;

            if is_auto_start_enabled && balance == 0 {
                return Err(ContractError::ZeroBalanceStreamStart);
            }

            commission += token.commission_on_create;

            if is_locked {
                // For locked streams we take all commission at the beginning
                let (_, calculated_commission) = token.apply_commission(balance);
                commission += calculated_commission;
            }

            token.collected_commission += commission;

            self.dao.tokens.insert(token_account_id.clone(), token);
        } else {
            if creator.deposit < self.dao.commission_unlisted {
                return Err(ContractError::InsufficientNearBalance {
                    requested: self.dao.commission_unlisted,
                    left: creator.deposit,
                });
            }
            creator.deposit -= self.dao.commission_unlisted;
        }

        if balance > MAX_AMOUNT {
            return Err(ContractError::ExceededMaxBalance {
                max_amount: MAX_AMOUNT,
            });
        }

        let cliff = if let Some(period) = cliff_period_sec {
            if !is_auto_start_enabled {
                return Err(ContractError::MustStartImmediately);
            }
            Some(env::block_timestamp() + TICKS_PER_SECOND * period as u64)
        } else {
            None
        };

        // Validations passed

        let mut stream = Stream::new(
            description,
            creator_id,
            owner_id,
            receiver_id,
            token_account_id,
            balance,
            tokens_per_sec,
            cliff,
            initial_balance,
            is_expirable,
            is_locked,
        );

        creator.total_streams_created += 1;
        creator.last_created_stream = Some(stream.id);
        self.save_account(creator)?;

        log!("{:?}", stream.id);

        self.process_action(&mut stream, ActionType::Init)?;

        self.stats_inc_stream_deposit(&stream.token_account_id, &balance, &commission);
        self.stats_inc_streams(
            &stream.token_account_id,
            Contract::is_aurora_address(&stream.owner_id)
                | Contract::is_aurora_address(&stream.receiver_id),
            is_listed,
        );

        if is_auto_start_enabled {
            self.process_action(&mut stream, ActionType::Start)?;
        }

        self.save_stream(stream)?;

        Ok(())
    }

    pub(crate) fn process_deposit(
        &mut self,
        token_account_id: AccountId,
        stream_id: CryptoHash,
        amount: Balance,
    ) -> Result<(), ContractError> {
        // NEP-141 forbids zero-token transfers, so this should never happen.
        assert_ne!(amount, 0);

        let stream_id = stream_id.into();
        let mut stream = self.extract_stream(&stream_id)?;
        if stream.status.is_terminated() {
            return Err(ContractError::StreamTerminated { stream_id });
        }

        if stream.is_locked {
            return Err(ContractError::StreamLocked {
                stream_id: stream.id,
            });
        }

        stream.update_cliff();

        if stream.cliff.is_some() {
            return Err(ContractError::CliffNotPassed {
                timestamp: stream.cliff.unwrap(),
            });
        }

        if stream.available_to_withdraw() == stream.balance
            && stream.balance > 0
            && stream.is_expirable
        {
            let action = self.process_action(
                &mut stream,
                ActionType::Stop {
                    reason: StreamFinishReason::FinishedBecauseCannotBeExtended,
                },
            )?;
            assert!(action.is_empty());
            self.save_stream(stream)?;
            return Err(ContractError::StreamExpired { stream_id });
        }

        if stream.token_account_id != token_account_id {
            return Err(ContractError::InvalidToken {
                expected: stream.token_account_id,
                received: token_account_id,
            });
        }

        if amount > MAX_AMOUNT || stream.balance + amount > MAX_AMOUNT {
            return Err(ContractError::ExceededMaxBalance {
                max_amount: MAX_AMOUNT,
            });
        }

        // Validations passed

        stream.balance += amount;
        stream.amount_to_push = amount;

        self.save_stream(stream)?;

        self.stats_inc_stream_deposit(&token_account_id, &amount, &0);

        Ok(())
    }

    pub fn process_start_stream(
        &mut self,
        sender_id: &AccountId,
        stream_id: CryptoHash,
    ) -> Result<(), ContractError> {
        let mut stream = self.extract_stream(&stream_id)?;

        if stream.is_locked {
            return Err(ContractError::StreamLocked {
                stream_id: stream.id,
            });
        }

        if stream.owner_id != *sender_id {
            return Err(ContractError::CallerIsNotStreamOwner {
                expected: stream.owner_id,
                received: sender_id.clone(),
            });
        }
        if stream.status != StreamStatus::Paused && stream.status != StreamStatus::Initialized {
            return Err(ContractError::CannotStartStream {
                stream_status: stream.status,
            });
        }
        if stream.balance == 0 {
            return Err(ContractError::ZeroBalanceStreamStart);
        }

        if env::prepaid_gas() - env::used_gas() < MIN_GAS_FOR_PROCESS_ACTION {
            return Err(ContractError::InsufficientGas {
                expected: MIN_GAS_FOR_PROCESS_ACTION,
                left: env::prepaid_gas() - env::used_gas(),
            });
        }

        // Validations passed

        assert!(self
            .process_action(&mut stream, ActionType::Start)?
            .is_empty());

        self.save_stream(stream)?;

        Ok(())
    }

    pub fn process_pause_stream(
        &mut self,
        sender_id: &AccountId,
        stream_id: CryptoHash,
    ) -> Result<Vec<Promise>, ContractError> {
        let mut stream = self.extract_stream(&stream_id)?;

        if stream.is_locked {
            return Err(ContractError::StreamLocked {
                stream_id: stream.id,
            });
        }

        if stream.owner_id != *sender_id && stream.receiver_id != *sender_id {
            return Err(ContractError::CallerIsNotStreamActor {
                owner: stream.owner_id,
                receiver: stream.receiver_id,
                caller: sender_id.clone(),
            });
        }
        if stream.status != StreamStatus::Active {
            return Err(ContractError::CannotPauseStream {
                stream_status: stream.status,
            });
        }

        stream.update_cliff();

        if stream.cliff.is_some() {
            return Err(ContractError::CliffNotPassed {
                timestamp: stream.cliff.unwrap(),
            });
        }

        if env::prepaid_gas() - env::used_gas() < MIN_GAS_FOR_PROCESS_ACTION {
            return Err(ContractError::InsufficientGas {
                expected: MIN_GAS_FOR_PROCESS_ACTION,
                left: env::prepaid_gas() - env::used_gas(),
            });
        }

        // Validations passed

        let promises = self.process_action(&mut stream, ActionType::Pause)?;

        self.save_stream(stream)?;

        Ok(promises)
    }

    pub fn process_stop_stream(
        &mut self,
        sender_id: &AccountId,
        stream_id: CryptoHash,
    ) -> Result<Vec<Promise>, ContractError> {
        let mut stream = self.extract_stream(&stream_id)?;

        if stream.is_locked {
            return Err(ContractError::StreamLocked {
                stream_id: stream.id,
            });
        }

        if stream.owner_id != *sender_id && stream.receiver_id != *sender_id {
            return Err(ContractError::CallerIsNotStreamActor {
                owner: stream.owner_id,
                receiver: stream.receiver_id,
                caller: sender_id.clone(),
            });
        }
        if stream.status.is_terminated() {
            return Err(ContractError::CannotStopStream {
                stream_status: stream.status,
            });
        }

        let reason = if stream.owner_id == *sender_id {
            StreamFinishReason::StoppedByOwner
        } else {
            StreamFinishReason::StoppedByReceiver
        };

        stream.update_cliff();

        if env::prepaid_gas() - env::used_gas() < MIN_GAS_FOR_PROCESS_ACTION {
            return Err(ContractError::InsufficientGas {
                expected: MIN_GAS_FOR_PROCESS_ACTION,
                left: env::prepaid_gas() - env::used_gas(),
            });
        }

        // Validations passed

        let promises = self.process_action(&mut stream, ActionType::Stop { reason })?;

        self.save_stream(stream)?;

        Ok(promises)
    }

    pub fn process_withdraw(
        &mut self,
        sender_id: &AccountId,
        stream_id: CryptoHash,
    ) -> Result<Vec<Promise>, ContractError> {
        let mut stream = self.extract_stream(&stream_id)?;

        let receiver_view = self.view_account(&stream.receiver_id)?;

        if receiver_view.id != *sender_id && !receiver_view.is_cron_allowed {
            return Err(ContractError::CronCallsForbidden {
                received: receiver_view.id,
            });
        }

        if stream.status != StreamStatus::Active {
            return Err(ContractError::CannotWithdraw {
                stream_status: stream.status,
            });
        }

        stream.update_cliff();

        if stream.cliff.is_some() {
            return Err(ContractError::CliffNotPassed {
                timestamp: stream.cliff.unwrap(),
            });
        }

        if env::prepaid_gas() - env::used_gas() < MIN_GAS_FOR_PROCESS_ACTION {
            return Err(ContractError::InsufficientGas {
                expected: MIN_GAS_FOR_PROCESS_ACTION,
                left: env::prepaid_gas() - env::used_gas(),
            });
        }

        // Validations passed

        let promises = self.process_action(&mut stream, ActionType::Withdraw)?;

        self.save_stream(stream)?;

        Ok(promises)
    }

    pub fn process_change_receiver(
        &mut self,
        sender_id: &AccountId,
        stream_id: CryptoHash,
        receiver_id: AccountId,
        storage_balance_needed: Balance,
    ) -> Result<Vec<Promise>, ContractError> {
        let promises = self.process_withdraw(sender_id, stream_id)?;

        let mut stream = self.extract_stream(&stream_id)?;

        if stream.status != StreamStatus::Active {
            // Inactive stream won't be transferred
            self.save_stream(stream)?;
            return Ok(promises);
        }

        if stream.is_locked {
            return Err(ContractError::StreamLocked {
                stream_id: stream.id,
            });
        }

        let mut receiver = if let Ok(account) = self.extract_account(&receiver_id) {
            account
        } else {
            // Charge for account creation
            let token = self.dao.get_token_or_unlisted(&stream.token_account_id);
            if token.is_listed {
                if stream.balance < token.commission_on_create {
                    let balance = stream.balance;
                    stream.balance = 0;
                    let action = self.process_action(
                        &mut stream,
                        ActionType::Stop {
                            reason: StreamFinishReason::FinishedWhileTransferred,
                        },
                    )?;
                    assert!(action.is_empty());
                    self.save_stream(stream)?;

                    self.stats_withdraw(&token, 0, balance);
                    return Ok(promises);
                }
                stream.balance -= token.commission_on_create;
                self.stats_withdraw(&token, 0, token.commission_on_create);
            } else {
                // Charge in NEAR
                assert!(
                    env::attached_deposit()
                        >= self.dao.commission_unlisted + storage_balance_needed
                );
                self.stats_inc_account_deposit(self.dao.commission_unlisted, false);
            }
            self.create_account_if_not_exist(&receiver_id)?;
            self.extract_account(&receiver_id)?
        };

        let mut sender = self.extract_account(sender_id)?;

        assert!(sender.active_incoming_streams.remove(&stream_id));
        assert!(receiver.active_incoming_streams.insert(&stream_id));

        self.save_account(sender)?;
        self.save_account(receiver)?;

        stream.receiver_id = receiver_id;
        self.save_stream(stream)?;
        Ok(promises)
    }
}
