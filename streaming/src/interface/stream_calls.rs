use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn start_stream(&mut self, stream_id: Base58CryptoHash) {
        assert_one_yocto();
        self.process_start_stream(&env::signer_account_id(), stream_id.into())
            .unwrap()
    }

    #[payable]
    pub fn pause_stream(&mut self, stream_id: Base58CryptoHash) -> Vec<Promise> {
        assert_one_yocto();
        self.process_pause_stream(&env::signer_account_id(), stream_id.into())
            .unwrap()
    }

    #[payable]
    pub fn stop_stream(&mut self, stream_id: Base58CryptoHash) -> Vec<Promise> {
        assert_one_yocto();
        self.process_stop_stream(&env::signer_account_id(), stream_id.into())
            .unwrap()
    }

    #[payable]
    pub fn withdraw(&mut self, stream_ids: Vec<Base58CryptoHash>) -> Vec<Promise> {
        assert_one_yocto();

        stream_ids
            .iter()
            .map(|&stream_id| {
                self.process_withdraw(&env::signer_account_id(), stream_id.into())
                    .unwrap()
            })
            .flatten()
            .collect()
    }

    #[payable]
    pub fn change_receiver(
        &mut self,
        stream_id: Base58CryptoHash,
        receiver_id: AccountId,
    ) -> Vec<Promise> {
        let stream_id = stream_id.into();
        let stream_view = self.view_stream(&stream_id).unwrap();

        // In this case we expect that predecessor must be a NFT contract
        // which is called by holder of the NFT that streams tokens.
        assert_eq!(env::signer_account_id(), stream_view.receiver_id);

        // TODO assert that env::predecessor_account_id() is in DAO list of approved NFTs
        // TODO #11 and enable
        assert!(false);

        let token = self
            .dao
            .get_token_or_unlisted(&stream_view.token_account_id);

        // TODO explain why attached deposit is needed at the point
        let deposit_needed = if Contract::is_aurora_address(&stream_view.receiver_id) {
            // Receiver is at aurora, need no payment for storage deposit
            ONE_YOCTO
        } else {
            token.storage_balance_needed
        };
        assert!(env::attached_deposit() >= deposit_needed);

        self.process_change_receiver(
            &stream_view.receiver_id,
            stream_id.into(),
            receiver_id,
            deposit_needed,
        )
        .unwrap()
    }
}
