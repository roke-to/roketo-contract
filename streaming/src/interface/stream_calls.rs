use crate::*;

#[near_bindgen]
impl Contract {
    #[handle_result]
    #[payable]
    pub fn start_stream(&mut self, stream_id: Base58CryptoHash) -> Result<(), ContractError> {
        check_deposit(ONE_YOCTO)?;
        // Stream calls may be delegated to 3rd parties.
        // That's why it uses env::predecessor_account_id() here and below.
        self.start_stream_op(&env::predecessor_account_id(), stream_id.into())
    }

    #[handle_result]
    #[payable]
    pub fn change_description(
        &mut self,
        stream_id: Base58CryptoHash,
        new_description: String,
    ) -> Result<Vec<Promise>, ContractError> {
        check_deposit(ONE_YOCTO)?;
        self.change_description_op(stream_id.into(), new_description)
    }

    #[handle_result]
    #[payable]
    pub fn pause_stream(
        &mut self,
        stream_id: Base58CryptoHash,
    ) -> Result<Vec<Promise>, ContractError> {
        check_deposit(ONE_YOCTO)?;
        self.pause_stream_op(&env::predecessor_account_id(), stream_id.into())
    }

    #[handle_result]
    #[payable]
    pub fn stop_stream(
        &mut self,
        stream_id: Base58CryptoHash,
    ) -> Result<Vec<Promise>, ContractError> {
        check_deposit(ONE_YOCTO)?;
        self.stop_stream_op(&env::predecessor_account_id(), stream_id.into())
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
            .map(|stream_id| self.withdraw_op(&env::predecessor_account_id(), stream_id.into()))
            .collect::<Result<Vec<Vec<Promise>>, ContractError>>()?
            .into_iter()
            .flatten()
            .collect())
    }

    #[handle_result]
    #[payable]
    pub fn nft_change_receiver(
        &mut self,
        stream_id: Base58CryptoHash,
        receiver_id: AccountId,
    ) -> Result<Vec<Promise>, ContractError> {
        let stream_id = stream_id.into();
        let stream_view = self.view_stream(&stream_id)?;

        // In this case we expect that predecessor must be a NFT contract
        // which is called by holder of the NFT that streams tokens.
        assert_eq!(env::signer_account_id(), stream_view.receiver_id);

        if !self
            .dao
            .approved_nfts
            .contains(&env::predecessor_account_id())
        {
            return Err(ContractError::NFTNotApproved {
                account_id: env::predecessor_account_id(),
            });
        }

        let token = self.dao.get_token(&stream_view.token_account_id);

        let deposit_needed = if is_aurora_address(&stream_view.receiver_id) {
            // Receiver is at aurora, need no payment for storage deposit
            ONE_YOCTO
        } else {
            // Receiver is at NEAR, we'd rather pay for storage deposit
            // than allow a new user to lose tokens.
            token.storage_balance_needed
        };
        check_deposit(deposit_needed)?;

        self.change_receiver_op(
            &stream_view.receiver_id,
            stream_id.into(),
            receiver_id,
            deposit_needed,
        )
    }
}
