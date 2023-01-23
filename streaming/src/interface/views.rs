use crate::*;

#[derive(Deserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct AccountView {
    pub active_incoming_streams: u32,
    pub active_outgoing_streams: u32,
    pub inactive_incoming_streams: u32,
    pub inactive_outgoing_streams: u32,

    pub total_incoming: HashMap<AccountId, U128>,
    pub total_outgoing: HashMap<AccountId, U128>,
    pub total_received: HashMap<AccountId, U128>,

    #[serde(with = "u128_dec_format")]
    pub deposit: Balance,

    #[serde(with = "u128_dec_format")]
    pub stake: Balance,

    pub last_created_stream: Option<Base58CryptoHash>,
    pub is_cron_allowed: bool,
}

#[near_bindgen]
impl Contract {
    pub fn get_stats(self) -> Stats {
        let mut stats: Stats = self.stats.get().unwrap().into();
        stats.total_dao_tokens = stats.dao_tokens.len() as _;
        stats.total_accounts = self.accounts.len() as _;
        stats.total_streams = self.streams.len() as _;
        stats
    }

    pub fn get_dao(self) -> Dao {
        self.dao
    }

    pub fn get_token(self, token_account_id: AccountId) -> (Token, Option<TokenStats>) {
        (
            self.dao.get_token(&token_account_id),
            (Stats::from(self.stats.get().unwrap()))
                .dao_tokens
                .remove(&token_account_id),
        )
    }

    #[handle_result]
    pub fn get_stream(self, stream_id: Base58CryptoHash) -> Result<Stream, ContractError> {
        self.view_stream(&stream_id.into())
    }

    #[handle_result]
    pub fn get_account(
        self,
        account_id: AccountId,
        only_if_exist: Option<bool>,
    ) -> Result<AccountView, ContractError> {
        self.view_account(&account_id, only_if_exist.unwrap_or(false))
            .map(|a| AccountView {
                active_incoming_streams: a.active_incoming_streams.len() as _,
                active_outgoing_streams: a.active_outgoing_streams.len() as _,
                inactive_incoming_streams: a.inactive_incoming_streams.len() as _,
                inactive_outgoing_streams: a.inactive_outgoing_streams.len() as _,

                total_incoming: self
                    .dao
                    .tokens
                    .keys()
                    .map(|k| (k.clone(), U128(*a.total_incoming.get(k).unwrap_or(&0))))
                    .collect(),
                total_outgoing: self
                    .dao
                    .tokens
                    .keys()
                    .map(|k| (k.clone(), U128(*a.total_outgoing.get(k).unwrap_or(&0))))
                    .collect(),
                total_received: self
                    .dao
                    .tokens
                    .keys()
                    .map(|k| (k.clone(), U128(*a.total_received.get(k).unwrap_or(&0))))
                    .collect(),

                deposit: a.deposit,
                stake: a.stake,
                last_created_stream: a.last_created_stream.map(|w| w.into()),
                is_cron_allowed: a.is_cron_allowed,
            })
    }

    #[handle_result]
    pub fn get_account_incoming_streams(
        self,
        account_id: AccountId,
        from: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Vec<Stream>, ContractError> {
        let from = from.unwrap_or(0);
        let limit = limit.unwrap_or(DEFAULT_VIEW_STREAMS_LIMIT);
        let account = self.view_account(&account_id, false)?;
        Ok(self.collect_account_data(
            &account.active_incoming_streams,
            &account.inactive_incoming_streams,
            from,
            limit,
        ))
    }

    #[handle_result]
    pub fn get_account_outgoing_streams(
        self,
        account_id: AccountId,
        from: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Vec<Stream>, ContractError> {
        let from = from.unwrap_or(0);
        let limit = limit.unwrap_or(DEFAULT_VIEW_STREAMS_LIMIT);
        let account = self.view_account(&account_id, false)?;
        Ok(self.collect_account_data(
            &account.active_outgoing_streams,
            &account.inactive_outgoing_streams,
            from,
            limit,
        ))
    }

    #[handle_result]
    pub fn get_streams(
        &self,
        from: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Vec<Stream>, ContractError> {
        let from = from.unwrap_or(0);
        let limit = limit.unwrap_or(DEFAULT_VIEW_STREAMS_LIMIT);
        Ok((from..min(self.streams.len() as _, from + limit))
            .map(|i| self.streams.values_as_vector().get(i as _).unwrap().into())
            .collect())
    }

    #[handle_result]
    pub fn get_account_ft(
        self,
        account_id: AccountId,
        token_account_id: AccountId,
    ) -> Result<(U128, U128, U128), ContractError> {
        let account = self.view_account(&account_id, false)?;
        Ok((
            (*account.total_incoming.get(&token_account_id).unwrap_or(&0)).into(),
            (*account.total_outgoing.get(&token_account_id).unwrap_or(&0)).into(),
            (*account.total_received.get(&token_account_id).unwrap_or(&0)).into(),
        ))
    }
}

impl Contract {
    fn collect_account_data(
        &self,
        active_streams: &UnorderedSet<StreamId>,
        inactive_streams: &UnorderedSet<StreamId>,
        from: u32,
        limit: u32,
    ) -> Vec<Stream> {
        (from..min(active_streams.len() as _, from + limit))
            .map(|i| {
                self.streams
                    .get(&active_streams.as_vector().get(i as _).unwrap())
                    .unwrap()
                    .into()
            })
            .chain(
                (from..min(inactive_streams.len() as _, from + limit)).map(|i| {
                    self.streams
                        .get(&inactive_streams.as_vector().get(i as _).unwrap())
                        .unwrap()
                        .into()
                }),
            )
            .collect()
    }
}
