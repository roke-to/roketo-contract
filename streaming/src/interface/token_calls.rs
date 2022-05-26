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
    pub owner_id: AccountId,
    pub receiver_id: AccountId,
    pub tokens_per_sec: Balance,
    pub cliff_period_sec: Option<u32>,
    pub is_auto_start_enabled: Option<bool>,
    pub is_expirable: Option<bool>,
    pub is_locked: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum AuroraOperationalRequest {
    AccountDeposit,
    StartStream { stream_id: Base58CryptoHash },
    PauseStream { stream_id: Base58CryptoHash },
    StopStream { stream_id: Base58CryptoHash },
    Withdraw { stream_id: Base58CryptoHash },
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
                        let value = self.dao.eth_near_ratio.mult_safe(amount.into());
                        self.account_deposit(sender_id, value).unwrap();
                        // TODO process collected commission
                        self.stats_inc_account_deposit(value, true);
                        return PromiseOrValue::Value(U128::from(0));
                    }
                    AuroraOperationalRequest::StartStream { stream_id } => {
                        // TODO check owner
                        self.start_stream_op(&sender_id, (*stream_id).into())
                            .map(|_| vec![])
                    }
                    AuroraOperationalRequest::PauseStream { stream_id } => {
                        // TODO check owner
                        // TODO cover storage deposit
                        // in Aurora->NEAR calls with attached eth
                        self.pause_stream_op(&sender_id, (*stream_id).into())
                    }
                    AuroraOperationalRequest::StopStream { stream_id } => {
                        // TODO check owner
                        // TODO cover storage deposit
                        // in Aurora->NEAR calls with attached eth
                        self.stop_stream_op(&sender_id, (*stream_id).into())
                    }
                    AuroraOperationalRequest::Withdraw { stream_id } => {
                        // TODO cover storage deposit
                        // in Aurora->NEAR calls with attached eth
                        self.withdraw_op(&sender_id, (*stream_id).into())
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

        let token_account_id = env::predecessor_account_id();
        let key: Result<TransferCallRequest, _> = serde_json::from_str(&msg);
        if key.is_err() {
            log!("cannot parse message {:?}, error {:?}", msg, key);
            // return everything back
            return PromiseOrValue::Value(amount);
        }
        match key.unwrap() {
            TransferCallRequest::Stake => {
                assert_eq!(token_account_id, self.dao.utility_token_id);
                let mut sender = self.extract_account(&sender_id).unwrap();
                sender.stake += u128::from(amount);
                self.save_account(sender).unwrap();
                PromiseOrValue::Value(U128::from(0))
            }
            TransferCallRequest::Create { request } => {
                match self.create_stream_op(
                    request.description,
                    sender_id,
                    request.owner_id,
                    request.receiver_id,
                    token_account_id,
                    amount.into(),
                    request.tokens_per_sec,
                    request.cliff_period_sec,
                    request.is_auto_start_enabled,
                    request.is_expirable,
                    request.is_locked,
                ) {
                    Ok(()) => PromiseOrValue::Value(U128::from(0)),
                    Err(err) => panic!("error on stream creation, {:?}", err),
                }
            }
            TransferCallRequest::Deposit { stream_id } => {
                match self.deposit_op(token_account_id, stream_id.into(), amount.into()) {
                    Ok(()) => PromiseOrValue::Value(U128::from(0)),
                    Err(ContractError::StreamExpired { stream_id }) => {
                        log!("stream expired, {:?}", stream_id);
                        // return everything back
                        PromiseOrValue::Value(amount)
                    }
                    Err(err) => panic!("error on stream depositing, {:?}", err),
                }
            }
        }
    }
}
