use crate::*;

#[near_bindgen]
impl Contract {
    #[handle_result]
    #[payable]
    pub fn start_stream(&mut self, stream_id: Base58CryptoHash) -> Result<(), ContractError> {
        check_deposit(ONE_YOCTO)?;
        // Stream calls may be delegated to 3rd parties.
        // That's why it uses env::predecessor_account_id() here and below.
        self.process_start_stream(&env::predecessor_account_id(), stream_id.into())
    }

    #[handle_result]
    #[payable]
    pub fn pause_stream(
        &mut self,
        stream_id: Base58CryptoHash,
    ) -> Result<Vec<Promise>, ContractError> {
        check_deposit(ONE_YOCTO)?;
        self.process_pause_stream(&env::predecessor_account_id(), stream_id.into())
    }

    #[handle_result]
    #[payable]
    pub fn stop_stream(
        &mut self,
        stream_id: Base58CryptoHash,
    ) -> Result<Vec<Promise>, ContractError> {
        check_deposit(ONE_YOCTO)?;
        self.process_stop_stream(&env::predecessor_account_id(), stream_id.into())
    }

    #[handle_result]
    #[payable]
    pub fn withdraw(
        &mut self,
        stream_ids: Vec<Base58CryptoHash>,
    ) -> Result<Vec<Promise>, ContractError> {
        check_deposit(ONE_YOCTO)?;

        Ok(stream_ids
            .into_iter()
            .map(|stream_id| {
                self.process_withdraw(&env::predecessor_account_id(), stream_id.into())
            })
            .collect::<Result<Vec<Vec<Promise>>, ContractError>>()?
            .into_iter()
            .flatten()
            .collect())
    }

    #[handle_result]
    #[payable]
    pub fn change_receiver(
        &mut self,
        stream_id: Base58CryptoHash,
        receiver_id: AccountId,
    ) -> Result<Vec<Promise>, ContractError> {
        let stream_id = stream_id.into();
        let stream_view = self.view_stream(&stream_id)?;

        // In this case we expect that predecessor must be a NFT contract
        // which is called by holder of the NFT that streams tokens.
        assert_eq!(env::signer_account_id(), stream_view.receiver_id);

        // TODO assert that env::predecessor_account_id() is in DAO list of approved NFTs
        // TODO #11 and enable
        assert!(false);

        let token = self.dao.get_token(&stream_view.token_account_id);

        // TODO explain why attached deposit is needed at the point
        let deposit_needed = if Contract::is_aurora_address(&stream_view.receiver_id) {
            // Receiver is at aurora, need no payment for storage deposit
            ONE_YOCTO
        } else {
            token.storage_balance_needed
        };
        check_deposit(deposit_needed)?;

        self.process_change_receiver(
            &stream_view.receiver_id,
            stream_id.into(),
            receiver_id,
            deposit_needed,
        )
    }
}
