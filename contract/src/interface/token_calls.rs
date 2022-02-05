use crate::*;

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum TransferCallRequest {
    Stake,
    Create { request: CreateRequest },
    Deposit { stream_id: Base58CryptoHash },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CreateRequest {
    pub description: Option<String>,
    pub receiver_id: AccountId,
    pub tokens_per_sec: Balance,
    pub is_auto_start_enabled: Option<bool>,
    pub is_expirable: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum AuroraOperationalRequest {
    AccountDeposit,
    StartStream {
        stream_id: Base58CryptoHash,
    },
    PauseStream {
        stream_id: Base58CryptoHash,
    },
    StopStream {
        stream_id: Base58CryptoHash,
    },
    Withdraw {
        stream_id: Base58CryptoHash,
        is_storage_deposit_needed: bool,
    },
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    // NEP-141 interface
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        if env::predecessor_account_id() == Contract::aurora_account_id() {
            // Try to parse as an Aurora operational request
            let key: Result<AuroraOperationalRequest, _> = serde_json::from_str(&msg);
            if key.is_ok() {
                let res = match key.as_ref().unwrap() {
                    AuroraOperationalRequest::AccountDeposit => {
                        let value = self.dao.eth_near_ratio.mult(amount.into());
                        self.account_deposit(env::signer_account_id(), value)
                            .unwrap();
                        self.dao
                            .tokens
                            .entry(Contract::aurora_account_id())
                            .and_modify(|e| e.collected_commission += value);
                        self.stats_inc_account_deposit(&value, true);
                        return PromiseOrValue::Value(U128::from(0));
                    }
                    AuroraOperationalRequest::StartStream { stream_id } => self
                        .process_start_stream((*stream_id).into())
                        .map(|_| vec![]),
                    AuroraOperationalRequest::PauseStream { stream_id } => {
                        // TODO cover storage deposit
                        // in Aurora->NEAR calls with attached eth
                        self.process_pause_stream((*stream_id).into())
                    }
                    AuroraOperationalRequest::StopStream { stream_id } => {
                        // TODO cover storage deposit
                        // in Aurora->NEAR calls with attached eth
                        self.process_stop_stream((*stream_id).into())
                    }
                    AuroraOperationalRequest::Withdraw {
                        stream_id,
                        is_storage_deposit_needed,
                    } => {
                        // TODO cover storage deposit
                        // in Aurora->NEAR calls with attached eth
                        self.process_withdraw((*stream_id).into(), *is_storage_deposit_needed)
                    }
                };
                return match res {
                    Ok(promises) => {
                        log!(
                            "Success on {:?}, {:?} promises started",
                            key,
                            promises.len()
                        );
                        PromiseOrValue::Value(U128::from(0))
                    }
                    Err(err) => {
                        panic!("Error {:?} on {:?}", err, key);
                    }
                };
            } else {
                // It still can be a TransferCallRequest
            }
        }

        let token = self
            .dao
            .get_token_or_unlisted(&env::predecessor_account_id());
        let key: Result<TransferCallRequest, _> = serde_json::from_str(&msg);
        if key.is_err() {
            log!("cannot parse message {:?}, error {:?}", msg, key);
            // return everything back
            return PromiseOrValue::Value(amount);
        }
        match key.unwrap() {
            TransferCallRequest::Stake => {
                assert_eq!(token.account_id, self.dao.utility_token_id);
                let mut account = self.extract_account(&env::signer_account_id()).unwrap();
                account.stake += u128::from(amount);
                self.save_account(account).unwrap();
                PromiseOrValue::Value(U128::from(0))
            }
            TransferCallRequest::Create { request } => {
                match self.process_create_stream(
                    sender_id,
                    request.description,
                    request.receiver_id,
                    token,
                    amount.into(),
                    request.tokens_per_sec,
                    request.is_auto_start_enabled,
                    request.is_expirable,
                ) {
                    Ok(()) => PromiseOrValue::Value(U128::from(0)),
                    Err(err) => panic!("error on stream creation, {:?}", err),
                }
            }
            TransferCallRequest::Deposit { stream_id } => {
                match self.process_deposit(token, stream_id.into(), amount.into()) {
                    Ok(()) => PromiseOrValue::Value(U128::from(0)),
                    Err(err) => panic!("error on stream depositing, {:?}", err),
                }
            }
        }
    }
}
