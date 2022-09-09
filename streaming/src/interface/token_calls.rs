use crate::*;

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum TransferCallRequest {
    Stake,
    Create { request: CreateRequest },
    Deposit { stream_id: Base58CryptoHash },
    BatchCreate { requests: Vec<CreateRequest> },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CreateRequest {
    pub description: Option<String>,
    pub owner_id: AccountId,
    pub receiver_id: AccountId,
    pub tokens_per_sec: U128,
    pub cliff_period_sec: Option<u32>,
    pub is_auto_start_enabled: Option<bool>,
    pub is_expirable: Option<bool>,
    pub is_locked: Option<bool>,
    pub balance: Option<U128>,
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
        if env::predecessor_account_id() == aurora_account_id() {
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

        // Deserialize TransferCallRequest from attached msg with json.
        let request = match serde_json::from_str::<TransferCallRequest>(&msg) {
            Ok(req) => req,
            Err(e) => {
                log!("cannot parse message {:?}, error {:?}", msg, e);
                // return everything back
                return PromiseOrValue::Value(amount);
            }
        };

        match request {
            TransferCallRequest::BatchCreate { requests } => {
                // Vaidate that enough tokens were transferred.
                // Skip requests without balance.
                let streams_total_balance: u128 = requests
                    .iter()
                    .filter_map(|req| req.balance)
                    .try_fold(0u128, |acc, b| acc.checked_add(b.0))
                    .expect("total streams balance overflow");
                assert!(
                    streams_total_balance <= amount.0,
                    "not enough tokens transferred to create all streams"
                );

                let mut refund = amount.0;

                for (i, req) in requests.into_iter().enumerate() {
                    let initial_balance = if let Some(b) = req.balance {
                        b.0
                    } else {
                        log!("Stream #{} create request must contrain 'balance'", i);
                        continue;
                    };

                    // If stream creation failed, just skip it and refund balance later.
                    if let Err(err) = self.create_stream_op(
                        req.description,
                        sender_id.clone(),
                        req.owner_id,
                        req.receiver_id,
                        token_account_id.clone(),
                        initial_balance,
                        req.tokens_per_sec.0,
                        req.cliff_period_sec,
                        req.is_auto_start_enabled,
                        req.is_expirable,
                        req.is_locked,
                    ) {
                        log!("Stream #{} create error: {:?}", i, err);
                    } else {
                        refund = refund.checked_sub(initial_balance).expect(
                            "overflow, transferred amount is less than total streams balance",
                        );
                        log!("Stream #{} created", i);
                    }
                }
                PromiseOrValue::Value(U128::from(refund))
            }
            TransferCallRequest::Stake => {
                assert_eq!(token_account_id, self.dao.utility_token_id);
                let mut sender = self.extract_account(&sender_id).unwrap();
                sender.stake += u128::from(amount);
                self.save_account(sender).unwrap();
                PromiseOrValue::Value(U128::from(0))
            }
            TransferCallRequest::Create { request } => {
                let mut refund = 0;

                // If balance field is present in request, use it as initial balance
                // and refund remaining tokens.
                // Else use transferred amount as initial balance.
                let initial_balance = if let Some(balance) = request.balance {
                    assert!(
                        balance.0 <= amount.0,
                        "not enough tokens transferred to cover initial balance"
                    );
                    refund = amount
                        .0
                        .checked_sub(balance.0)
                        .expect("unreachable 'cause of the previous check");
                    balance.0
                } else {
                    amount.0
                };
                match self.create_stream_op(
                    request.description,
                    sender_id.clone(),
                    request.owner_id,
                    request.receiver_id,
                    token_account_id,
                    initial_balance,
                    request.tokens_per_sec.0,
                    request.cliff_period_sec,
                    request.is_auto_start_enabled,
                    request.is_expirable,
                    request.is_locked,
                ) {
                    Ok(()) => PromiseOrValue::Value(U128::from(refund)),
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
