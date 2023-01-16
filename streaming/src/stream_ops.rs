use crate::*;

impl Contract {
    pub(crate) fn create_stream_op(
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

        let token = self.dao.get_token(&token_account_id);
        let mut commission = 0;

        if token.is_payment {
            // Take commission as DAO proposed
            if balance < token.commission_on_create {
                return Err(ContractError::InsufficientDeposit {
                    expected: token.commission_on_create,
                    received: balance,
                });
            }
            balance -= token.commission_on_create;
            commission += token.commission_on_create;

            if is_auto_start_enabled {
                if balance == 0 {
                    return Err(ContractError::ZeroBalanceStreamStart);
                }
                if is_locked {
                    // For locked streams we take all commission when the stream is started
                    let (_, calculated_commission) = token.apply_commission(balance);
                    commission += calculated_commission;
                }
            }
        } else {
            if creator.deposit < self.dao.commission_non_payment_ft {
                return Err(ContractError::InsufficientNearBalance {
                    requested: self.dao.commission_non_payment_ft,
                    left: creator.deposit,
                });
            }
            creator.deposit -= self.dao.commission_non_payment_ft;
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
            self.streams.len(),
            description,
            creator_id,
            owner_id,
            receiver_id,
            token_account_id,
            balance,
            tokens_per_sec,
            cliff,
            is_expirable,
            is_locked,
        );

        creator.total_streams_created += 1;
        creator.last_created_stream = Some(stream.id);
        self.save_account(creator)?;

        self.process_action(&mut stream, ActionType::Init)?;

        self.stats_inc_stream_deposit(&stream.token_account_id, &balance, &commission);
        self.stats_inc_streams(
            &stream.token_account_id,
            is_aurora_address(&stream.owner_id) | is_aurora_address(&stream.receiver_id),
            token.is_payment,
        );

        if is_auto_start_enabled {
            self.process_action(&mut stream, ActionType::Start)?;
        }

        self.ft_transfer_from_self(
            stream.token_account_id.clone(),
            self.finance_id.clone(),
            stream.balance,
        )?;

        // Covering storage needs from finance contract
        ext_finance::ext(self.finance_id.clone())
            .with_attached_deposit(ONE_YOCTO)
            .with_static_gas(Gas::ONE_TERA * 10)
            .streaming_storage_needs_transfer();

        self.save_stream(stream)?;

        Ok(())
    }

    pub(crate) fn deposit_op(
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

        if stream.is_locked && stream.status != StreamStatus::Initialized {
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

        self.ft_transfer_from_self(
            stream.token_account_id.clone(),
            self.finance_id.clone(),
            amount,
        )?;

        self.save_stream(stream)?;

        self.stats_inc_stream_deposit(&token_account_id, &amount, &0);

        Ok(())
    }

    pub fn start_stream_op(
        &mut self,
        sender_id: &AccountId,
        stream_id: CryptoHash,
    ) -> Result<(), ContractError> {
        let mut stream = self.extract_stream(&stream_id)?;

        if stream.is_locked && stream.status != StreamStatus::Initialized {
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

        // Validations passed

        if stream.is_locked {
            let token = self.dao.get_token(&stream.token_account_id);
            if token.is_payment {
                // For locked streams we take all commission when the stream is started
                let (_, commission) = token.apply_commission(stream.balance);
                self.stats_inc_stream_deposit(&stream.token_account_id, &0, &commission);
            }
        };

        assert!(self
            .process_action(&mut stream, ActionType::Start)?
            .is_empty());

        self.save_stream(stream)?;

        Ok(())
    }

    pub fn pause_stream_op(
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

        // Validations passed

        let promises = self.process_action(&mut stream, ActionType::Pause)?;

        self.save_stream(stream)?;

        Ok(promises)
    }

    pub fn stop_stream_op(
        &mut self,
        sender_id: &AccountId,
        stream_id: CryptoHash,
    ) -> Result<Vec<Promise>, ContractError> {
        let mut stream = self.extract_stream(&stream_id)?;

        if stream.is_locked && stream.status != StreamStatus::Initialized {
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

        // Validations passed

        let promises = self.process_action(&mut stream, ActionType::Stop { reason })?;

        self.save_stream(stream)?;

        Ok(promises)
    }

    pub fn withdraw_op(
        &mut self,
        sender_id: &AccountId,
        stream_id: CryptoHash,
    ) -> Result<Vec<Promise>, ContractError> {
        let mut stream = self.extract_stream(&stream_id)?;

        let receiver_view = self.view_account(&stream.receiver_id, true)?;

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

        // Validations passed

        let promises = self.process_action(&mut stream, ActionType::Withdraw)?;

        self.save_stream(stream)?;

        Ok(promises)
    }

    pub fn change_description_op(
        &mut self,
        sender_id: &AccountId,
        stream_id: CryptoHash,
        new_description: Option<String>,
    ) -> Result<(), ContractError> {
        let mut stream = self.extract_stream(&stream_id)?;

        if stream.status.is_terminated() {
            return Err(ContractError::StreamTerminated {
                stream_id: stream_id,
            });
        }

        if stream.is_locked {
            return Err(ContractError::StreamLocked {
                stream_id: stream_id,
            });
        }

        if stream.owner_id != *sender_id {
            return Err(ContractError::CallerIsNotStreamOwner {
                expected: stream.owner_id,
                received: sender_id.clone(),
            });
        }
        if let Some(text) = &new_description {
            if text.len() > MAX_DESCRIPTION_LEN {
                return Err(ContractError::DescriptionTooLong {
                    max_description_len: MAX_DESCRIPTION_LEN,
                    received: text.len(),
                });
            }
        }
        stream.description = new_description;
        self.save_stream(stream)
    }

    pub fn change_receiver_op(
        &mut self,
        prev_receiver_id: &AccountId,
        stream_id: CryptoHash,
        new_receiver_id: AccountId,
        deposit_needed: Balance,
    ) -> Result<Vec<Promise>, ContractError> {
        let mut promises = self.withdraw_op(prev_receiver_id, stream_id)?;

        let mut stream = self.extract_stream(&stream_id)?;

        if stream.status.is_terminated() {
            return Err(ContractError::StreamTerminated {
                stream_id: stream.id,
            });
        }

        if stream.is_locked {
            return Err(ContractError::StreamLocked {
                stream_id: stream.id,
            });
        }

        let token = self.dao.get_token(&stream.token_account_id);

        let mut new_receiver = if let Ok(account) = self.extract_account(&new_receiver_id) {
            account
        } else {
            // TODO revisit understanding of charging for change receiver
            // Charge for account creation
            if token.is_payment {
                if stream.balance <= token.commission_on_transfer {
                    let balance = stream.balance;
                    stream.balance = 0;
                    let action = self.process_action(
                        &mut stream,
                        ActionType::Stop {
                            reason: StreamFinishReason::FinishedWhileTransferred,
                        },
                    )?;
                    // No transfer tokens actions should appear at the point.
                    // All tokens have been charged to previous holder + commission.
                    assert!(action.is_empty());
                    self.save_stream(stream)?;

                    self.stats_withdraw(&token, 0, balance);
                    return Ok(promises);
                }
                stream.balance -= token.commission_on_transfer;
                self.stats_withdraw(&token, 0, token.commission_on_transfer);
            } else {
                // Charge in NEAR
                check_deposit(self.dao.commission_non_payment_ft + deposit_needed)?;
                self.stats_inc_account_deposit(self.dao.commission_non_payment_ft, false);
            }
            self.create_account_if_not_exist(&new_receiver_id)?;
            self.extract_account(&new_receiver_id)?
        };

        let mut prev_receiver = self.extract_account(prev_receiver_id)?;

        check_integrity(prev_receiver.active_incoming_streams.remove(&stream_id))?;
        check_integrity(new_receiver.active_incoming_streams.insert(&stream_id))?;

        prev_receiver
            .total_incoming
            .entry(stream.token_account_id.clone())
            .and_modify(|e| *e -= stream.tokens_per_sec);
        new_receiver
            .total_incoming
            .entry(stream.token_account_id.clone())
            .and_modify(|e| *e += stream.tokens_per_sec);

        self.save_account(prev_receiver)?;
        self.save_account(new_receiver)?;

        let storage_deposit_promise = ext_storage_management::ext(token.account_id)
            .with_attached_deposit(deposit_needed)
            .with_static_gas(token.gas_for_storage_deposit)
            .storage_deposit(Some(new_receiver_id.clone()), Some(true));
        promises.push(storage_deposit_promise);

        stream.receiver_id = new_receiver_id;
        self.save_stream(stream)?;

        Ok(promises)
    }
}
