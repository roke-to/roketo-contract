#[cfg(test)]
mod tests {
    use crate::*;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::{env, test_utils::VMContextBuilder, testing_env};

    fn finance_id() -> AccountId {
        "finance.near".parse().unwrap()
    }
    fn dao_id() -> AccountId {
        "dao.near".parse().unwrap()
    }
    fn utility_token_id() -> AccountId {
        "utilitytoken.near".parse().unwrap()
    }

    fn new_stream() -> Stream {
        Stream {
            id: env::sha256(&[44, 55, 66]).as_slice().try_into().unwrap(),
            description: Some("blah".to_string()),
            creator_id: carol(),
            owner_id: alice(),
            receiver_id: bob(),
            token_account_id: "token.near".parse().unwrap(),
            timestamp_created: env::block_timestamp(),
            last_action: env::block_timestamp(),
            balance: 1_000_000_000_000_000_000_000_000_000, // 1e27
            tokens_per_sec: 1_000_000_000_000_000_000_000_000, // 1e24
            status: StreamStatus::Active,
            tokens_total_withdrawn: 0,
            cliff: None,
            is_expirable: true,
            is_locked: false,
            available_to_withdraw_by_formula: 0,
        }
    }

    #[test]
    fn test_save_extract_stream() {
        let mut contract = Contract::new(dao_id(), finance_id(), utility_token_id(), 18);
        let stream_id = new_stream().id;
        assert!(contract.extract_stream(&stream_id).is_err());
        assert!(contract.save_stream(new_stream()).is_ok());
        assert!(contract.save_stream(new_stream()).is_err());
        assert!(contract.extract_stream(&stream_id).is_ok());
        assert!(contract.extract_stream(&stream_id).is_err());
    }

    #[test]
    fn test_create_stream() {
        let mut contract = Contract::new(dao_id(), finance_id(), utility_token_id(), 18);
        testing_env!(VMContextBuilder::new().signer_account_id(carol()).build());
        let stream = new_stream();
        assert_eq!(
            contract.create_stream_op(
                stream.description,
                carol(),
                stream.owner_id,
                stream.receiver_id,
                stream.token_account_id,
                stream.balance,
                stream.tokens_per_sec,
                None,
                None,
                None,
                None,
            ),
            Err(ContractError::InsufficientNearBalance {
                requested: 100000000000000000000000u128,
                left: 0,
            })
        );

        testing_env!(VMContextBuilder::new()
            .signer_account_id(carol())
            .predecessor_account_id(carol())
            .attached_deposit(DEFAULT_COMMISSION_NON_PAYMENT_FT)
            .build());
        contract.account_deposit_near().unwrap();
        let stream = new_stream();
        testing_env!(VMContextBuilder::new()
            .signer_account_id(carol())
            .predecessor_account_id(stream.token_account_id.clone())
            .build());
        assert!(contract
            .create_stream_op(
                stream.description,
                carol(),
                stream.owner_id,
                stream.receiver_id,
                stream.token_account_id,
                stream.balance,
                stream.tokens_per_sec,
                None,
                None,
                None,
                None,
            )
            .is_ok());
    }

    #[test]
    fn test_create_stream_to_aurora() {
        let mut contract = Contract::new(dao_id(), finance_id(), utility_token_id(), 18);
        testing_env!(VMContextBuilder::new()
            .signer_account_id(carol())
            .predecessor_account_id(carol())
            .attached_deposit(DEFAULT_COMMISSION_NON_PAYMENT_FT)
            .build());
        contract.account_deposit_near().unwrap();
        let mut stream = new_stream();
        stream.receiver_id =
            AccountId::new_unchecked("f5cfbc74057c610c8ef151a439252680ac68c6dc".to_string());
        testing_env!(VMContextBuilder::new()
            .signer_account_id(carol())
            .predecessor_account_id(stream.token_account_id.clone())
            .build());
        assert!(contract
            .create_stream_op(
                stream.description,
                carol(),
                stream.owner_id,
                stream.receiver_id,
                stream.token_account_id,
                stream.balance,
                stream.tokens_per_sec,
                None,
                None,
                None,
                None,
            )
            .is_ok());
    }
}
