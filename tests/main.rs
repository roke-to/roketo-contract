// TODO need more tests
//
// promise then failure
// multiple streams same token
// multiple streams multiple token
// multiple withdraw
// multiple withdraw one fail revert
// stake/unstake
// stop reasons
// instant deposit
// dao calls
// exchanger calls
// access to stream actions
// test withdraw no storage deposit
// test stats
// streams are properly deleted from accounts
// dao token updated while streaming
// dao new token
// dao new token while streaming (unlisted -> listed)
// dao remove token
// unlisted tokens low decimals
// unlisted tokens high decimals
// unlisted tokens commissions
// deposit while streaming
// deposit invalid token into stream
// near->aurora transfers
// aurora create stream aurora
// aurora create stream listed
// aurora create stream unlisted
// aurora deposit
// aurora account ids / addresses
// aurora deployment + aurora tokens
// aurora ops
// exchanger take commission sanity
// test description
// test stream expiration
// locked unlisted
// locked with cliff
// locked deposit
// change receiver tests
// nft transfer custom tests
// multiple nfts
// nft several copies attach
// multiple nft transfers
// insufficient storage deposit tests
// nft stream finished while transfer
// nft stream finished while transfer unlisted
// nft stream finished while withdrawed

mod setup;

use crate::setup::*;

#[test]
fn test_init_env() {
    let e = Env::init();
    let _tokens = Tokens::init(&e);
    let _users = Users::init(&e);
}

#[test]
fn test_mint_tokens() {
    let e = Env::init();
    let tokens = Tokens::init(&e);
    let users = Users::init(&e);
    e.mint_tokens(&tokens, &users.alice, 100);
}

#[test]
fn test_dev_setup() {
    let e = Env::init();
    let tokens = Tokens::init(&e);
    e.setup_assets(&tokens);

    let dao = e.get_dao();
    assert_eq!(dao.tokens.len(), 5);

    let stats = e.get_stats();
    assert_eq!(stats.dao_tokens.len(), 5);

    let (_, s) = e.get_token(&tokens.aurora);
    assert!(s.is_some());

    let (_, s) = e.get_token(&tokens.aurora);
    assert!(s.is_some());

    let (_, s) = e.get_token(&tokens.dacha);
    assert!(s.is_none());
}

#[test]
fn test_finance_transfers() {
    let (e, tokens, users) = basic_setup();

    assert_eq!(e.get_balance(&tokens.wnear, &e.streaming.user_account), 0);
    assert_eq!(e.get_balance(&tokens.wnear, &e.finance), 0);

    let amount = d(101, 23);
    e.create_stream_ext_err(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        amount,
        d(1, 23),
        None,
        None,
        None,
        None,
        None,
    );

    assert_eq!(
        e.get_balance(&tokens.wnear, &e.streaming.user_account),
        d(1, 23)
    );
    assert_eq!(e.get_balance(&tokens.wnear, &e.finance), d(100, 23));
}

// Actual tests start here

#[test]
fn test_stream_sanity() {
    let (e, tokens, users) = basic_setup();

    let amount = d(101, 23);
    let stream_id = e.create_stream(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        amount,
        d(1, 23),
    );

    e.skip_time(100);

    let dao = e.get_dao();
    let dao_token = dao.tokens.get(&tokens.wnear.account_id()).unwrap();
    let amount_after_create = amount - dao_token.commission_on_create;
    let stream = e.get_stream(&stream_id);

    assert_eq!(stream.balance, amount_after_create);
    assert_eq!(stream.owner_id, users.alice.account_id());
    assert_eq!(stream.receiver_id, users.charlie.account_id());
    assert_eq!(stream.tokens_total_withdrawn, 0);
    assert_eq!(stream.status, StreamStatus::Active);

    e.stop_stream(&users.alice, &stream_id);

    let amount_after_stop =
        amount_after_create - dao_token.commission_coef.mult_safe(amount_after_create);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.balance, 0);
    assert_eq!(stream.owner_id, users.alice.account_id());
    assert_eq!(stream.receiver_id, users.charlie.account_id());
    assert_eq!(stream.tokens_total_withdrawn, amount_after_create);
    assert_eq!(
        stream.status,
        StreamStatus::Finished {
            reason: StreamFinishReason::FinishedNaturally
        }
    );

    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        amount_after_stop
    );
}

#[test]
fn test_stream_min_value() {
    let (e, tokens, users) = basic_setup();

    let amount = d(1, 6) + 3700;
    let stream_id = e.create_stream(
        &users.alice,
        &users.charlie,
        &tokens.nusdt,
        amount,
        MIN_STREAMING_SPEED,
    );

    // Zero token transfer
    e.withdraw(&users.charlie, &stream_id);
    let stream = e.get_stream(&stream_id);

    assert_eq!(stream.balance, 3700);
    assert_eq!(stream.tokens_total_withdrawn, 0);
    assert_eq!(stream.status, StreamStatus::Active);
    assert_eq!(e.get_balance(&tokens.nusdt, &users.charlie), 0);
    let stats = e.get_stats();
    let dao_token = stats.dao_tokens.get(&tokens.nusdt.account_id()).unwrap();
    assert_eq!(dao_token.total_commission_collected, d(1, 6));

    e.skip_time(1);
    e.withdraw(&users.charlie, &stream_id);
    let stream = e.get_stream(&stream_id);

    assert_eq!(stream.balance, 3700 - 1);
    assert_eq!(stream.tokens_total_withdrawn, 1);
    assert_eq!(stream.status, StreamStatus::Active);
    assert_eq!(e.get_balance(&tokens.nusdt, &users.charlie), 0);
    let stats = e.get_stats();
    let dao_token = stats.dao_tokens.get(&tokens.nusdt.account_id()).unwrap();
    assert_eq!(dao_token.total_commission_collected, d(1, 6) + 1);

    e.skip_time(150);
    e.withdraw(&users.charlie, &stream_id);
    let stream = e.get_stream(&stream_id);

    assert_eq!(stream.balance, 3700 - 1 - 150);
    assert_eq!(stream.tokens_total_withdrawn, 1 + 150);
    assert_eq!(stream.status, StreamStatus::Active);
    assert_eq!(e.get_balance(&tokens.nusdt, &users.charlie), 149);
    let stats = e.get_stats();
    let dao_token = stats.dao_tokens.get(&tokens.nusdt.account_id()).unwrap();
    assert_eq!(dao_token.total_commission_collected, d(1, 6) + 2);

    e.skip_time(10000);
    e.withdraw(&users.charlie, &stream_id);
    let stream = e.get_stream(&stream_id);

    assert_eq!(stream.balance, 0);
    assert_eq!(stream.tokens_total_withdrawn, 3700);
    assert_eq!(
        stream.status,
        StreamStatus::Finished {
            reason: StreamFinishReason::FinishedNaturally
        }
    );
    assert_eq!(e.get_balance(&tokens.nusdt, &users.charlie), 3700 - 6);
    let stats = e.get_stats();
    let dao_token = stats.dao_tokens.get(&tokens.nusdt.account_id()).unwrap();
    assert_eq!(dao_token.total_commission_collected, d(1, 6) + 6);
}

#[test]
fn test_stream_max_value() {
    let (e, tokens, users) = basic_setup();
    e.mint_ft(&tokens.wnear, &users.alice, MAX_AMOUNT);

    let dao = e.get_dao();
    let mut dao_token = dao.tokens.get(&tokens.wnear.account_id()).unwrap().clone();
    dao_token.commission_coef = SafeFloat {
        val: 1_000_000_000 - 1,
        pow: -9,
    };
    dao_token.commission_on_create = 0;
    e.dao_update_token(dao_token);

    let amount = MAX_AMOUNT;
    let stream_id = e.create_stream(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        amount,
        MAX_STREAMING_SPEED,
    );

    e.skip_time(60 * 60 * 24 * 365 * 100); // 100 years

    e.withdraw(&users.charlie, &stream_id);

    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.tokens_total_withdrawn, MAX_AMOUNT);
    assert_eq!(
        stream.status,
        StreamStatus::Finished {
            reason: StreamFinishReason::FinishedNaturally
        }
    );

    let dao = e.get_dao();
    let dao_token = dao.tokens.get(&tokens.wnear.account_id()).unwrap();
    let stats = e.get_stats();
    let stats_token = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(
        stats_token.total_commission_collected,
        dao_token.commission_coef.mult_safe(MAX_AMOUNT - 1) + 1
    );

    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        MAX_AMOUNT - stats_token.total_commission_collected
    );

    // prove that the test makes any sence
    assert_eq!(
        (MAX_AMOUNT / 1_000_000_000 * (1_000_000_000 - 1) - stats_token.total_commission_collected)
            / d(1, 18),
        0
    );
}

#[test]
fn test_stream_max_value_min_speed() {
    let (e, tokens, users) = basic_setup();
    e.mint_ft(&tokens.wnear, &users.alice, MAX_AMOUNT);

    let dao = e.get_dao();
    let mut dao_token = dao.tokens.get(&tokens.wnear.account_id()).unwrap().clone();
    dao_token.commission_coef = SafeFloat {
        val: 1_000_000_000 - 1,
        pow: -9,
    };
    dao_token.commission_on_create = 0;
    e.dao_update_token(dao_token);

    let amount = MAX_AMOUNT;
    let stream_id = e.create_stream(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        amount,
        MIN_STREAMING_SPEED,
    );

    let hund_years = 60 * 60 * 24 * 365 * 100; // 100 years
    e.skip_time(hund_years as u64);

    e.withdraw(&users.charlie, &stream_id);

    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.tokens_total_withdrawn, hund_years);
    assert_eq!(stream.status, StreamStatus::Active);

    let dao = e.get_dao();
    let dao_token = dao.tokens.get(&tokens.wnear.account_id()).unwrap();
    let stats = e.get_stats();
    let stats_token = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(
        stats_token.total_commission_collected,
        dao_token.commission_coef.mult_safe(hund_years - 1) + 1
    );

    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        hund_years - stats_token.total_commission_collected
    );
}

#[test]
fn test_incoming_outgoing_sanity() {
    let (e, tokens, users) = basic_setup();

    let stream_id = e.create_stream(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        d(1, 26), // 100 tokesn
        d(1, 25), //10 tokens per sec
    );

    let stream = e.get_stream(&stream_id);
    assert!(e.get_account_incoming_streams(&users.alice).len() == 0);
    assert!(e.get_account_incoming_streams(&users.charlie).len() == 1);
    assert!(e.get_account_outgoing_streams(&users.alice).len() == 1);
    assert!(e.get_account_outgoing_streams(&users.charlie).len() == 0);

    assert!(e.get_account_incoming_streams(&users.charlie)[0].id == stream.id);
    assert!(e.get_account_outgoing_streams(&users.alice)[0].id == stream.id);
}

#[test]
fn test_stream_start_pause_finished() {
    let (e, tokens, users) = basic_setup();

    let stream_id = e.create_stream(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        d(1, 26), // 100 tokesn
        d(1, 25), //10 tokens per sec
    );

    e.skip_time(20); // waiting 20 sec

    assert!(e.get_account_incoming_streams(&users.alice).len() == 0);
    assert!(e.get_account_incoming_streams(&users.charlie).len() == 1);
    assert!(e.get_account_outgoing_streams(&users.alice).len() == 1);
    assert!(e.get_account_outgoing_streams(&users.charlie).len() == 0);
    e.pause_stream(&users.charlie, &stream_id); // pause
    assert!(e.get_account_incoming_streams(&users.alice).len() == 0);
    assert!(e.get_account_incoming_streams(&users.charlie).len() == 0);
    assert!(e.get_account_outgoing_streams(&users.alice).len() == 0);
    assert!(e.get_account_outgoing_streams(&users.charlie).len() == 0);
}

#[test]
fn test_check_fixing_inactive_streams() {
    let (e, tokens, users) = basic_setup();

    let mut accounts = Vec::new();
    for i in 10..35 {
        let x: i32 = i;
        let acc_id = AccountId::new_unchecked(x.to_string());
        let acc = e.near.create_user(acc_id.clone(), d(1, 28));
        ft_storage_deposit(&e.near, &tokens.wnear.account_id(), &acc_id);
        e.mint_ft(&tokens.wnear, &acc, d(1000000, 24));
        accounts.push(acc);
    }
    assert!(accounts.len() == 25, "{}", accounts.len());
    let mut streams = Vec::new();
    let n = 20;
    for i in 0..n {
        let stream_id = e.create_stream(
            &accounts[i],
            &accounts[i + 1],
            &tokens.wnear,
            d(1, 26),
            d(1, 25),
        );
        streams.push(stream_id);
    }
    e.skip_time(20); // waiting 20 sec
    for i in 0..n {
        e.pause_stream(&accounts[i], &streams[i]);
    }
    //e.pause_stream(&accounts[19], &streams[19]);
    for i in 1..n {
        assert!(
            e.get_account_outgoing_streams(&accounts[i]).len() == 1,
            "{}",
            i
        );
        assert!(
            e.get_account_incoming_streams(&accounts[i]).len() == 1,
            "{}",
            i
        );
    }
    e.fixing_streams(
        e.near
            .create_user("rubinchik.near".parse().unwrap(), d(1, 26)),
    );
    for i in 0..n {
        assert!(
            e.get_account_outgoing_streams(&accounts[i]).len() == 0,
            "{}",
            i
        );
        assert!(
            e.get_account_incoming_streams(&accounts[i]).len() == 0,
            "{}",
            i
        );
    }
}

#[test]
fn test_stream_start_pause_stop() {
    let (e, tokens, users) = basic_setup();

    let amount = d(1, 24);

    let stream_id = e.create_stream_ext(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        amount,
        d(1, 20),
        None,
        None,
        Some(false),
        None,
        None,
    );

    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Initialized);

    e.skip_time(10);
    assert!(!e.withdraw_err(&users.charlie, &stream_id).is_ok());

    e.skip_time(10);
    assert!(!e.start_stream_err(&users.bob, &stream_id).is_ok());
    assert!(!e.start_stream_err(&users.charlie, &stream_id).is_ok());
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Initialized);

    e.skip_time(10);
    e.start_stream(&users.alice, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Active);

    e.skip_time(10);
    assert!(!e.pause_stream_err(&users.bob, &stream_id).is_ok());
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Active);

    e.skip_time(10);
    assert_eq!(e.get_balance(&tokens.wnear, &users.charlie), 0);
    e.pause_stream(&users.charlie, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Paused);
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        d(20, 20) / 250 * 249
    );
    assert_eq!(stream.balance, amount - d(1, 23) - d(20, 20));

    e.skip_time(10);
    assert!(!e.pause_stream_err(&users.alice, &stream_id).is_ok());
    assert!(!e.pause_stream_err(&users.bob, &stream_id).is_ok());
    assert!(!e.pause_stream_err(&users.charlie, &stream_id).is_ok());
    assert!(!e.start_stream_err(&users.bob, &stream_id).is_ok());
    assert!(!e.start_stream_err(&users.charlie, &stream_id).is_ok());
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Paused);

    e.skip_time(10);
    let last_alice_balance = e.get_balance(&tokens.wnear, &users.alice);
    assert!(!e.stop_stream_err(&users.bob, &stream_id).is_ok());
    e.stop_stream(&users.charlie, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(
        stream.status,
        StreamStatus::Finished {
            reason: StreamFinishReason::StoppedByReceiver
        }
    );
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        d(20, 20) / 250 * 249
    );
    assert_eq!(stream.balance, 0);
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.alice),
        last_alice_balance + (amount - d(1, 23) - d(20, 20))
    );

    e.skip_time(10);
    assert!(!e.pause_stream_err(&users.alice, &stream_id).is_ok());
    assert!(!e.pause_stream_err(&users.bob, &stream_id).is_ok());
    assert!(!e.pause_stream_err(&users.charlie, &stream_id).is_ok());
    assert!(!e.start_stream_err(&users.alice, &stream_id).is_ok());
    assert!(!e.start_stream_err(&users.bob, &stream_id).is_ok());
    assert!(!e.start_stream_err(&users.charlie, &stream_id).is_ok());
    assert!(!e.stop_stream_err(&users.alice, &stream_id).is_ok());
    assert!(!e.stop_stream_err(&users.bob, &stream_id).is_ok());
    assert!(!e.stop_stream_err(&users.charlie, &stream_id).is_ok());
    let stream = e.get_stream(&stream_id);
    assert_eq!(
        stream.status,
        StreamStatus::Finished {
            reason: StreamFinishReason::StoppedByReceiver
        }
    );
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        d(20, 20) / 250 * 249
    );
    assert_eq!(stream.balance, 0);
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.alice),
        last_alice_balance + (amount - d(1, 23) - d(20, 20))
    );
}

#[test]
fn test_stream_unlisted_sanity() {
    let (e, tokens, users) = basic_setup();

    let amount = d(1, 20);
    let token = tokens.dacha;

    let last_alice_balance = e.get_balance(&token, &users.alice);

    assert_eq!(
        e.create_stream_ext_err(
            &users.alice,
            &users.charlie,
            &token,
            amount,
            d(1, 16),
            None,
            None,
            None,
            None,
            None,
        ),
        U128(0)
    );
    assert_eq!(last_alice_balance, e.get_balance(&token, &users.alice));

    let dao = e.get_dao();

    assert!(!users
        .alice
        .function_call(
            e.streaming.contract.account_deposit_near(),
            MAX_GAS,
            dao.commission_unlisted - 1,
        )
        .is_ok());

    assert!(e
        .near
        .view_method_call(e.streaming.contract.get_account(users.alice.account_id()))
        .is_err());

    assert!(users
        .alice
        .function_call(
            e.streaming.contract.account_deposit_near(),
            MAX_GAS,
            dao.commission_unlisted,
        )
        .is_ok());

    assert!(e
        .near
        .view_method_call(e.streaming.contract.get_account(users.alice.account_id()))
        .is_ok());
    let account = e.get_account(&users.alice);
    assert_eq!(account.deposit, dao.commission_unlisted);

    let stream_id = e.create_stream(&users.alice, &users.charlie, &token, amount, d(1, 16));

    let account = e.get_account(&users.alice);
    assert_eq!(account.deposit, 0);
    assert_eq!(
        last_alice_balance - amount,
        e.get_balance(&token, &users.alice)
    );

    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Active);
    assert_eq!(stream.token_account_id, token.account_id());
}

#[test]
fn test_stream_unlisted_start_pause_stop() {
    let (e, tokens, users) = basic_setup();

    let amount = d(1, 20);
    let token = tokens.dacha;

    let dao = e.get_dao();
    e.account_deposit_near(&users.alice, dao.commission_unlisted);

    let stream_id = e.create_stream_ext(
        &users.alice,
        &users.charlie,
        &token,
        amount,
        d(1, 16),
        None,
        None,
        Some(false),
        None,
        None,
    );

    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Initialized);

    e.skip_time(10);
    assert!(!e.withdraw_err(&users.charlie, &stream_id).is_ok());

    e.skip_time(10);
    assert!(!e.start_stream_err(&users.bob, &stream_id).is_ok());
    assert!(!e.start_stream_err(&users.charlie, &stream_id).is_ok());
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Initialized);

    e.skip_time(10);
    e.start_stream(&users.alice, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Active);

    e.skip_time(10);
    assert!(!e.pause_stream_err(&users.bob, &stream_id).is_ok());
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Active);

    e.skip_time(10);
    assert_eq!(e.get_balance(&token, &users.charlie), 0);
    e.pause_stream(&users.charlie, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Paused);
    assert_eq!(e.get_balance(&token, &users.charlie), d(20, 16));
    assert_eq!(stream.balance, amount - d(20, 16));

    e.skip_time(10);
    assert!(!e.pause_stream_err(&users.alice, &stream_id).is_ok());
    assert!(!e.pause_stream_err(&users.bob, &stream_id).is_ok());
    assert!(!e.pause_stream_err(&users.charlie, &stream_id).is_ok());
    assert!(!e.start_stream_err(&users.bob, &stream_id).is_ok());
    assert!(!e.start_stream_err(&users.charlie, &stream_id).is_ok());
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Paused);

    e.skip_time(10);
    let last_alice_balance = e.get_balance(&token, &users.alice);
    assert!(!e.stop_stream_err(&users.bob, &stream_id).is_ok());
    e.stop_stream(&users.charlie, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(
        stream.status,
        StreamStatus::Finished {
            reason: StreamFinishReason::StoppedByReceiver
        }
    );
    assert_eq!(e.get_balance(&token, &users.charlie), d(20, 16));
    assert_eq!(stream.balance, 0);
    assert_eq!(
        e.get_balance(&token, &users.alice),
        last_alice_balance + (amount - d(20, 16))
    );

    e.skip_time(10);
    assert!(!e.pause_stream_err(&users.alice, &stream_id).is_ok());
    assert!(!e.pause_stream_err(&users.bob, &stream_id).is_ok());
    assert!(!e.pause_stream_err(&users.charlie, &stream_id).is_ok());
    assert!(!e.start_stream_err(&users.alice, &stream_id).is_ok());
    assert!(!e.start_stream_err(&users.bob, &stream_id).is_ok());
    assert!(!e.start_stream_err(&users.charlie, &stream_id).is_ok());
    assert!(!e.stop_stream_err(&users.alice, &stream_id).is_ok());
    assert!(!e.stop_stream_err(&users.bob, &stream_id).is_ok());
    assert!(!e.stop_stream_err(&users.charlie, &stream_id).is_ok());
    let stream = e.get_stream(&stream_id);
    assert_eq!(
        stream.status,
        StreamStatus::Finished {
            reason: StreamFinishReason::StoppedByReceiver
        }
    );
    assert_eq!(e.get_balance(&token, &users.charlie), d(20, 16));
    assert_eq!(stream.balance, 0);
    assert_eq!(
        e.get_balance(&token, &users.alice),
        last_alice_balance + (amount - d(20, 16))
    );
}

#[test]
fn test_withdraw_invalid() {
    let (e, tokens, users) = basic_setup();
    let token = tokens.wnear;
    let amount = d(1, 26);

    let stream_id_1 = e.create_stream(&users.alice, &users.charlie, &token, amount, d(1, 20));

    let stream_id_2 = e.create_stream(&users.bob, &users.dude, &token, amount, d(1, 20));

    e.skip_time(100);

    assert_eq!(e.get_balance(&token, &users.charlie), 0);
    assert_eq!(e.get_balance(&token, &users.dude), 0);

    // wrong actor
    assert!(!e
        .withdraw_ext_err(&users.charlie, &[&stream_id_1, &stream_id_2])
        .is_ok());
    assert!(!e
        .withdraw_ext_err(&users.charlie, &[&stream_id_2, &stream_id_1])
        .is_ok());

    assert_eq!(e.get_balance(&token, &users.charlie), 0);
    assert_eq!(e.get_balance(&token, &users.dude), 0);

    let stream_1 = e.get_stream(&stream_id_1);
    assert_eq!(stream_1.balance, d(1, 26) - d(1, 23));
    assert_eq!(stream_1.tokens_total_withdrawn, 0);
    assert_eq!(stream_1.status, StreamStatus::Active);

    let stream_2 = e.get_stream(&stream_id_2);
    assert_eq!(stream_2.balance, d(1, 26) - d(1, 23));
    assert_eq!(stream_2.tokens_total_withdrawn, 0);
    assert_eq!(stream_2.status, StreamStatus::Active);

    // should work
    assert!(e
        .withdraw_ext_err(&users.charlie, &[&stream_id_1, &stream_id_1])
        .is_ok());

    assert_eq!(
        e.get_balance(&token, &users.charlie),
        d(100, 20) / 250 * 249
    );
    assert_eq!(e.get_balance(&token, &users.dude), 0);

    let stream_1 = e.get_stream(&stream_id_1);
    assert_eq!(stream_1.balance, d(1, 26) - d(1, 23) - d(100, 20));
    assert_eq!(stream_1.tokens_total_withdrawn, d(100, 20));
    assert_eq!(stream_1.status, StreamStatus::Active);

    let stream_2 = e.get_stream(&stream_id_2);
    assert_eq!(stream_2.balance, d(1, 26) - d(1, 23));
    assert_eq!(stream_2.tokens_total_withdrawn, 0);
    assert_eq!(stream_2.status, StreamStatus::Active);

    // not enough gas
    assert!(!e
        .withdraw_ext_err(&users.dude, &vec![&stream_id_2; 100])
        .is_ok());

    assert_eq!(
        e.get_balance(&token, &users.charlie),
        d(100, 20) / 250 * 249
    );
    assert_eq!(e.get_balance(&token, &users.dude), 0);

    let stream_1 = e.get_stream(&stream_id_1);
    assert_eq!(stream_1.balance, d(1, 26) - d(1, 23) - d(100, 20));
    assert_eq!(stream_1.tokens_total_withdrawn, d(100, 20));
    assert_eq!(stream_1.status, StreamStatus::Active);

    let stream_2 = e.get_stream(&stream_id_2);
    assert_eq!(stream_2.balance, d(1, 26) - d(1, 23));
    assert_eq!(stream_2.tokens_total_withdrawn, 0);
    assert_eq!(stream_2.status, StreamStatus::Active);

    // ok
    assert!(e
        .withdraw_ext_err(&users.dude, &vec![&stream_id_2; 10])
        .is_ok());

    assert_eq!(
        e.get_balance(&token, &users.charlie),
        d(100, 20) / 250 * 249
    );
    assert_eq!(e.get_balance(&token, &users.dude), d(100, 20) / 250 * 249);

    let stream_1 = e.get_stream(&stream_id_1);
    assert_eq!(stream_1.balance, d(1, 26) - d(1, 23) - d(100, 20));
    assert_eq!(stream_1.tokens_total_withdrawn, d(100, 20));
    assert_eq!(stream_1.status, StreamStatus::Active);

    let stream_2 = e.get_stream(&stream_id_2);
    assert_eq!(stream_2.balance, d(1, 26) - d(1, 23) - d(100, 20));
    assert_eq!(stream_2.tokens_total_withdrawn, d(100, 20));
    assert_eq!(stream_2.status, StreamStatus::Active);
}

#[test]
fn test_withdraw_multiple() {
    let (e, tokens, users) = basic_setup();
    e.mint_tokens(&tokens, &users.charlie, 1000);
    let token = tokens.wnear;
    let amount = d(1, 26);

    let stream_id_1 = e.create_stream(&users.alice, &users.dude, &token, amount, d(1, 20));
    let stream_id_2 = e.create_stream(&users.bob, &users.dude, &token, amount, d(1, 21));
    let stream_id_3 = e.create_stream(&users.charlie, &users.dude, &token, amount, d(1, 22));

    e.skip_time(100);

    assert_eq!(e.get_balance(&token, &users.dude), 0);

    assert!(!e
        .withdraw_ext_err(&users.alice, &[&stream_id_1, &stream_id_2, &stream_id_3])
        .is_ok());
    assert!(!e
        .withdraw_ext_err(&users.bob, &[&stream_id_1, &stream_id_2, &stream_id_3])
        .is_ok());
    assert!(!e
        .withdraw_ext_err(&users.charlie, &[&stream_id_1, &stream_id_2, &stream_id_3])
        .is_ok());
    assert!(!e
        .withdraw_ext_err(&users.eve, &[&stream_id_1, &stream_id_2, &stream_id_3])
        .is_ok());

    // ok - withdraw all 3
    assert!(e
        .withdraw_ext_err(&users.dude, &[&stream_id_1, &stream_id_2, &stream_id_3])
        .is_ok());

    assert_eq!(
        e.get_balance(&token, &users.dude),
        (d(100, 20) + d(100, 21) + d(100, 22)) / 250 * 249
    );

    let stream_1 = e.get_stream(&stream_id_1);
    assert_eq!(stream_1.balance, d(1, 26) - d(1, 23) - d(100, 20));
    assert_eq!(stream_1.tokens_total_withdrawn, d(100, 20));
    assert_eq!(stream_1.status, StreamStatus::Active);

    let stream_2 = e.get_stream(&stream_id_2);
    assert_eq!(stream_2.balance, d(1, 26) - d(1, 23) - d(100, 21));
    assert_eq!(stream_2.tokens_total_withdrawn, d(100, 21));
    assert_eq!(stream_2.status, StreamStatus::Active);

    let stream_3 = e.get_stream(&stream_id_3);
    assert_eq!(stream_3.balance, d(1, 26) - d(1, 23) - d(100, 22));
    assert_eq!(stream_3.tokens_total_withdrawn, d(100, 22));
    assert_eq!(stream_3.status, StreamStatus::Active);

    e.skip_time(100);

    assert!(!e
        .withdraw_ext_err(&users.alice, &[&stream_id_3, &stream_id_2])
        .is_ok());
    assert!(!e
        .withdraw_ext_err(&users.bob, &[&stream_id_3, &stream_id_2])
        .is_ok());
    assert!(!e
        .withdraw_ext_err(&users.charlie, &[&stream_id_3, &stream_id_2])
        .is_ok());
    assert!(!e
        .withdraw_ext_err(&users.eve, &[&stream_id_3, &stream_id_2])
        .is_ok());

    // ok - withdraw only 2 of 3
    assert!(e
        .withdraw_ext_err(&users.dude, &[&stream_id_3, &stream_id_2])
        .is_ok());

    assert_eq!(
        e.get_balance(&token, &users.dude),
        (d(100, 20) + d(200, 21) + d(200, 22)) / 250 * 249
    );

    let stream_1 = e.get_stream(&stream_id_1);
    assert_eq!(stream_1.balance, d(1, 26) - d(1, 23) - d(100, 20));
    assert_eq!(stream_1.tokens_total_withdrawn, d(100, 20));
    assert_eq!(stream_1.status, StreamStatus::Active);

    let stream_2 = e.get_stream(&stream_id_2);
    assert_eq!(stream_2.balance, d(1, 26) - d(1, 23) - d(200, 21));
    assert_eq!(stream_2.tokens_total_withdrawn, d(200, 21));
    assert_eq!(stream_2.status, StreamStatus::Active);

    let stream_3 = e.get_stream(&stream_id_3);
    assert_eq!(stream_3.balance, d(1, 26) - d(1, 23) - d(200, 22));
    assert_eq!(stream_3.tokens_total_withdrawn, d(200, 22));
    assert_eq!(stream_3.status, StreamStatus::Active);
}

#[test]
fn test_withdraw_multiple_allow_cron() {
    let (e, tokens, users) = basic_setup();
    let token = tokens.wnear;
    let amount = d(1, 26);

    let stream_id_1 = e.create_stream(&users.alice, &users.charlie, &token, amount, d(1, 20));
    let stream_id_2 = e.create_stream(&users.alice, &users.dude, &token, amount, d(1, 21));
    let stream_id_3 = e.create_stream(&users.alice, &users.eve, &token, amount, d(1, 22));

    e.skip_time(100);

    assert_eq!(e.get_balance(&token, &users.charlie), 0);
    assert_eq!(e.get_balance(&token, &users.dude), 0);
    assert_eq!(e.get_balance(&token, &users.eve), 0);

    assert!(!e
        .withdraw_ext_err(&users.eve, &[&stream_id_1, &stream_id_2, &stream_id_3])
        .is_ok());

    e.account_update_cron_flag(&users.charlie, true);
    e.account_update_cron_flag(&users.dude, true);

    assert!(!e
        .withdraw_ext_err(&users.alice, &[&stream_id_1, &stream_id_2, &stream_id_3])
        .is_ok());
    assert!(!e
        .withdraw_ext_err(&users.bob, &[&stream_id_1, &stream_id_2, &stream_id_3])
        .is_ok());
    assert!(!e
        .withdraw_ext_err(&users.charlie, &[&stream_id_1, &stream_id_2, &stream_id_3])
        .is_ok());
    assert!(!e
        .withdraw_ext_err(&users.dude, &[&stream_id_1, &stream_id_2, &stream_id_3])
        .is_ok());

    // ok
    assert!(e
        .withdraw_ext_err(&users.eve, &[&stream_id_1, &stream_id_2, &stream_id_3])
        .is_ok());

    assert_eq!(
        e.get_balance(&token, &users.charlie),
        d(100, 20) / 250 * 249
    );
    assert_eq!(e.get_balance(&token, &users.dude), d(100, 21) / 250 * 249);
    assert_eq!(e.get_balance(&token, &users.eve), d(100, 22) / 250 * 249);

    let stream_1 = e.get_stream(&stream_id_1);
    assert_eq!(stream_1.balance, d(1, 26) - d(1, 23) - d(100, 20));
    assert_eq!(stream_1.tokens_total_withdrawn, d(100, 20));
    assert_eq!(stream_1.status, StreamStatus::Active);

    let stream_2 = e.get_stream(&stream_id_2);
    assert_eq!(stream_2.balance, d(1, 26) - d(1, 23) - d(100, 21));
    assert_eq!(stream_2.tokens_total_withdrawn, d(100, 21));
    assert_eq!(stream_2.status, StreamStatus::Active);

    let stream_3 = e.get_stream(&stream_id_3);
    assert_eq!(stream_3.balance, d(1, 26) - d(1, 23) - d(100, 22));
    assert_eq!(stream_3.tokens_total_withdrawn, d(100, 22));
    assert_eq!(stream_3.status, StreamStatus::Active);
}

#[test]
fn test_dao_unlist_list() {
    let (e, tokens, users) = basic_setup();
    let token = tokens.wnear;
    let amount = d(1, 26);

    let stream_id = e.create_stream(&users.alice, &users.charlie, &token, amount, d(1, 20));

    e.skip_time(100);

    e.withdraw(&users.charlie, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.balance, amount - d(1, 23) - d(100, 20));
    assert_eq!(stream.tokens_total_withdrawn, d(100, 20));
    assert_eq!(stream.status, StreamStatus::Active);
    assert_eq!(
        e.get_balance(&token, &users.charlie),
        d(100, 20) / 250 * 249
    );

    e.skip_time(100);

    let dao = e.get_dao();
    let mut dao_token = dao.tokens.get(&token.account_id()).unwrap().clone();
    dao_token.is_listed = false;
    e.dao_update_token(dao_token);

    e.skip_time(100);

    e.withdraw(&users.charlie, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.balance, amount - d(1, 23) - d(300, 20));
    assert_eq!(stream.tokens_total_withdrawn, d(300, 20));
    assert_eq!(stream.status, StreamStatus::Active);
    assert_eq!(
        e.get_balance(&token, &users.charlie),
        d(100, 20) / 250 * 249 + d(200, 20)
    );

    e.skip_time(100);

    let dao = e.get_dao();
    let mut dao_token = dao.tokens.get(&token.account_id()).unwrap().clone();
    dao_token.is_listed = true;
    e.dao_update_token(dao_token);

    e.skip_time(100);

    e.withdraw(&users.charlie, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.balance, amount - d(1, 23) - d(500, 20));
    assert_eq!(stream.tokens_total_withdrawn, d(500, 20));
    assert_eq!(stream.status, StreamStatus::Active);
    assert_eq!(
        e.get_balance(&token, &users.charlie),
        d(300, 20) / 250 * 249 + d(200, 20)
    );
}

#[test]
fn test_stream_myself() {
    let (e, tokens, users) = basic_setup();
    let token = tokens.wnear;
    let amount = d(1, 24);

    let last_alice_balance = e.get_balance(&token, &users.alice);

    assert_eq!(
        e.create_stream_ext_err(
            &users.alice,
            &users.alice,
            &token,
            amount,
            d(1, 16),
            None,
            None,
            None,
            None,
            None,
        ),
        U128(0)
    );
    assert_eq!(last_alice_balance, e.get_balance(&token, &users.alice));
}

#[test]
fn test_stats_sanity() {
    let (e, tokens, users) = basic_setup();

    let stats = e.get_stats();
    assert_eq!(stats.total_streams, 0);
    assert_eq!(stats.total_active_streams, 0);
    assert_eq!(stats.total_accounts, 0);
    assert_eq!(stats.dao_tokens.len(), 5);
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(t.total_deposit, 0);
    assert_eq!(t.tvl, 0);
    assert_eq!(t.transferred, 0);
    assert_eq!(t.refunded, 0);
    assert_eq!(t.total_commission_collected, 0);
    assert_eq!(stats.last_update_time, 0);

    let amount = d(1, 24);

    let stream_id = e.create_stream(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        amount,
        d(1, 20),
    );

    let stats = e.get_stats();
    assert_eq!(stats.total_streams, 1);
    assert_eq!(stats.total_active_streams, 1);
    assert_eq!(stats.total_accounts, 2);
    assert_eq!(stats.dao_tokens.len(), 5);
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(t.total_deposit, d(1, 24));
    assert_eq!(t.tvl, d(1, 24) - d(1, 23));
    assert_eq!(t.transferred, 0);
    assert_eq!(t.refunded, 0);
    assert_eq!(t.total_commission_collected, d(1, 23));
    assert_eq!(stats.last_update_time, 0);

    e.skip_time(100);
    e.withdraw(&users.charlie, &stream_id);

    let stats = e.get_stats();
    assert_eq!(stats.total_streams, 1);
    assert_eq!(stats.total_active_streams, 1);
    assert_eq!(stats.total_accounts, 2);
    assert_eq!(stats.dao_tokens.len(), 5);
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(t.total_deposit, d(1, 24));
    assert_eq!(t.tvl, d(1, 24) - d(1, 23) - d(100, 20));
    assert_eq!(t.transferred, d(100, 20) / 250 * 249);
    assert_eq!(t.refunded, 0);
    assert_eq!(t.total_commission_collected, d(1, 23) + d(100, 20) / 250);
    assert_eq!(stats.last_update_time, d(100, 9) as u64);

    e.skip_time(100);
    e.stop_stream(&users.alice, &stream_id);

    let stats = e.get_stats();
    assert_eq!(stats.total_streams, 1);
    assert_eq!(stats.total_active_streams, 0);
    assert_eq!(stats.total_accounts, 2);
    assert_eq!(stats.dao_tokens.len(), 5);
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(t.total_deposit, d(1, 24));
    assert_eq!(t.tvl, 0);
    assert_eq!(t.transferred, d(200, 20) / 250 * 249);
    assert_eq!(t.refunded, amount - d(1, 23) - d(200, 20));
    assert_eq!(t.total_commission_collected, d(1, 23) + d(200, 20) / 250);
    assert_eq!(stats.last_update_time, d(200, 9) as u64);
}

#[test]
fn test_dao_collect_commission_sanity() {
    let (e, tokens, users) = basic_setup();

    let stats = e.get_stats();
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(t.total_commission_collected, 0);

    let amount = d(1, 24);

    let stream_id = e.create_stream(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        amount,
        d(1, 20),
    );

    let stats = e.get_stats();
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(t.total_commission_collected, d(1, 23));

    e.skip_time(100);
    e.withdraw(&users.charlie, &stream_id);

    let stats = e.get_stats();
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(t.total_commission_collected, d(1, 23) + d(100, 20) / 250);

    e.skip_time(100);
    e.stop_stream(&users.alice, &stream_id);

    let stats = e.get_stats();
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(t.total_commission_collected, d(1, 23) + d(200, 20) / 250);
}

#[test]
fn test_stream_cliff_sanity() {
    let (e, tokens, users) = basic_setup();

    let amount = d(1, 24);

    assert_eq!(
        e.create_stream_ext_err(
            &users.alice,
            &users.charlie,
            &tokens.wnear,
            amount,
            d(1, 20),
            None,
            Some(100),
            Some(false),
            None,
            None,
        ),
        U128(0)
    );

    let stream_id = e.create_stream_ext(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        amount,
        d(1, 20),
        None,
        Some(100),
        None,
        None,
        None,
    );

    assert_eq!(
        e.deposit_err(&users.alice, &stream_id, &tokens.wnear, d(1, 24)),
        U128(0)
    );

    e.skip_time(99);
    assert!(!e.withdraw_err(&users.charlie, &stream_id).is_ok());
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Active);
    assert_eq!(stream.cliff, Some(100 * 1_000_000_000));

    assert_eq!(
        e.deposit_err(&users.alice, &stream_id, &tokens.wnear, d(1, 24)),
        U128(0)
    );

    e.skip_time(1);
    e.withdraw(&users.charlie, &stream_id);

    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Active);
    assert_eq!(stream.cliff, None);
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        d(100, 20) / 250 * 249
    );
    assert_eq!(stream.balance, amount - d(1, 23) - d(100, 20));

    e.skip_time(1);
    e.withdraw(&users.charlie, &stream_id);

    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Active);
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        d(101, 20) / 250 * 249
    );
    assert_eq!(stream.balance, amount - d(1, 23) - d(101, 20));

    assert_eq!(
        e.deposit_err(&users.alice, &stream_id, &tokens.wnear, d(1, 24)),
        U128(d(1, 24))
    );
}

#[test]
fn test_stream_withdraw_after_cliff() {
    let (e, tokens, users) = basic_setup();

    let amount = d(1, 24);

    let stream_id = e.create_stream_ext(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        amount,
        d(1, 20),
        None,
        Some(100),
        None,
        None,
        None,
    );

    e.skip_time(200);
    e.withdraw(&users.charlie, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Active);
    assert_eq!(stream.cliff, None);
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        d(200, 20) / 250 * 249
    );
    assert_eq!(stream.balance, amount - d(1, 23) - d(200, 20));

    let stats = e.get_stats();
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(t.total_commission_collected, d(1, 23) + d(200, 20) / 250);

    e.skip_time(200);
    e.withdraw(&users.charlie, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Active);
    assert_eq!(stream.cliff, None);
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        d(400, 20) / 250 * 249
    );
    assert_eq!(stream.balance, amount - d(1, 23) - d(400, 20));

    let stats = e.get_stats();
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(t.total_commission_collected, d(1, 23) + d(400, 20) / 250);
}

#[test]
fn test_stream_cliff_pause() {
    let (e, tokens, users) = basic_setup();

    let amount = d(1, 24);

    let stream_id = e.create_stream_ext(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        amount,
        d(1, 20),
        None,
        Some(100),
        None,
        None,
        None,
    );

    e.skip_time(60);
    assert!(!e.pause_stream_err(&users.charlie, &stream_id).is_ok());
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Active);
    assert_eq!(e.get_balance(&tokens.wnear, &users.charlie), 0);
    assert_eq!(stream.balance, amount - d(1, 23));
    assert!(!e.withdraw_err(&users.charlie, &stream_id).is_ok());

    e.skip_time(50);
    e.pause_stream(&users.alice, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.cliff, None);
    assert_eq!(stream.status, StreamStatus::Paused);
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        d(110, 20) / 250 * 249
    );
    assert_eq!(stream.balance, amount - d(1, 23) - d(110, 20));
    assert!(!e.withdraw_err(&users.charlie, &stream_id).is_ok());
}

#[test]
fn test_stream_cliff_stop() {
    let (e, tokens, users) = basic_setup();

    let amount = d(1, 24);

    let initial_balance = e.get_balance(&tokens.wnear, &users.alice);

    let stream_id = e.create_stream_ext(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        amount,
        d(1, 20),
        None,
        Some(100),
        None,
        None,
        None,
    );

    assert_eq!(
        e.get_balance(&tokens.wnear, &users.alice),
        initial_balance - d(1, 24)
    );

    e.skip_time(60);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.cliff, Some(100 * 1_000_000_000));
    assert_eq!(stream.status, StreamStatus::Active);
    e.stop_stream(&users.alice, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.cliff, Some(100 * 1_000_000_000));
    assert_eq!(
        stream.status,
        StreamStatus::Finished {
            reason: StreamFinishReason::StoppedByOwner
        }
    );
    let stats = e.get_stats();
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(t.total_commission_collected, d(1, 23) + d(60, 20) / 250);
    assert_eq!(e.get_balance(&tokens.wnear, &users.charlie), 0);
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.alice),
        initial_balance - d(1, 23) - d(60, 20) / 250
    );
    assert_eq!(stream.balance, 0);
}

#[test]
fn test_stream_stop_after_cliff() {
    let (e, tokens, users) = basic_setup();

    let amount = d(1, 24);

    let initial_balance = e.get_balance(&tokens.wnear, &users.alice);

    let stream_id = e.create_stream_ext(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        amount,
        d(1, 20),
        None,
        Some(100),
        None,
        None,
        None,
    );

    assert_eq!(
        e.get_balance(&tokens.wnear, &users.alice),
        initial_balance - d(1, 24)
    );

    e.skip_time(120);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.cliff, Some(100 * 1_000_000_000));
    assert_eq!(stream.status, StreamStatus::Active);
    e.stop_stream(&users.alice, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.cliff, None);
    assert_eq!(
        stream.status,
        StreamStatus::Finished {
            reason: StreamFinishReason::StoppedByOwner
        }
    );
    let stats = e.get_stats();
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(t.total_commission_collected, d(1, 23) + d(120, 20) / 250);
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        d(120, 20) / 250 * 249
    );
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.alice),
        initial_balance - d(1, 23) - d(120, 20)
    );
    assert_eq!(stream.balance, 0);
}

#[test]
fn test_stream_locked_sanity() {
    let (e, tokens, users) = basic_setup();

    let amount = d(1, 24);

    let initial_balance = e.get_balance(&tokens.wnear, &users.alice);

    let stream_id = e.create_stream_ext(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        amount,
        d(1, 20),
        None,
        None,
        None,
        None,
        Some(true),
    );

    for user in [
        &users.alice,
        &users.bob,
        &users.charlie,
        &users.dude,
        &users.eve,
        &e.near,
        &e.dao,
        &e.streaming.user_account,
    ] {
        assert!(!e.start_stream_err(user, &stream_id).is_ok());
        assert!(!e.pause_stream_err(user, &stream_id).is_ok());
        assert!(!e.stop_stream_err(user, &stream_id).is_ok());
    }
    for user in [&users.alice, &users.bob] {
        assert_eq!(
            e.deposit_err(user, &stream_id, &tokens.wnear, d(1, 20)),
            U128(0)
        );
    }

    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Active);
    assert_eq!(e.get_balance(&tokens.wnear, &users.charlie), 0);
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.alice),
        initial_balance - d(1, 24)
    );
    assert_eq!(stream.balance, d(1, 24) - d(1, 23));

    e.skip_time(100);
    e.withdraw(&users.charlie, &stream_id);
    for user in [
        &users.alice,
        &users.bob,
        &users.charlie,
        &users.dude,
        &users.eve,
        &e.near,
        &e.dao,
        &e.streaming.user_account,
    ] {
        assert!(!e.start_stream_err(user, &stream_id).is_ok());
        assert!(!e.pause_stream_err(user, &stream_id).is_ok());
        assert!(!e.stop_stream_err(user, &stream_id).is_ok());
    }
    for user in [&users.alice, &users.bob] {
        assert_eq!(
            e.deposit_err(user, &stream_id, &tokens.wnear, d(1, 20)),
            U128(0)
        );
    }
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Active);

    e.skip_time(10000);
    e.withdraw(&users.charlie, &stream_id);
    for user in [
        &users.alice,
        &users.bob,
        &users.charlie,
        &users.dude,
        &users.eve,
        &e.near,
        &e.dao,
        &e.streaming.user_account,
    ] {
        assert!(!e.start_stream_err(user, &stream_id).is_ok());
        assert!(!e.pause_stream_err(user, &stream_id).is_ok());
        assert!(!e.stop_stream_err(user, &stream_id).is_ok());
    }
    for user in [&users.alice, &users.bob] {
        assert_eq!(
            e.deposit_err(user, &stream_id, &tokens.wnear, d(1, 20)),
            U128(0)
        );
    }
    let stream = e.get_stream(&stream_id);
    assert_eq!(
        stream.status,
        StreamStatus::Finished {
            reason: StreamFinishReason::FinishedNaturally
        }
    );

    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        (d(1, 24) - d(1, 23)) / 250 * 249
    );
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.alice),
        initial_balance - d(1, 24)
    );
    assert_eq!(stream.balance, 0);

    let stats = e.get_stats();
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(
        t.total_commission_collected,
        d(1, 23) + (d(1, 24) - d(1, 23)) / 250
    );
}

#[test]
fn test_stream_locked_commissions() {
    let (e, tokens, users) = basic_setup();

    let amount = d(1, 24);

    let initial_balance = e.get_balance(&tokens.wnear, &users.alice);

    let stats = e.get_stats();
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(t.total_commission_collected, 0);

    let stream_id = e.create_stream_ext(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        amount,
        d(1, 20),
        None,
        None,
        None,
        None,
        Some(true),
    );

    let stats = e.get_stats();
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(
        t.total_commission_collected,
        d(1, 23) + (d(1, 24) - d(1, 23)) / 250
    );

    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Active);
    assert_eq!(e.get_balance(&tokens.wnear, &users.charlie), 0);
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.alice),
        initial_balance - d(1, 24)
    );
    assert_eq!(stream.balance, d(1, 24) - d(1, 23));

    e.skip_time(100);
    e.withdraw(&users.charlie, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(stream.status, StreamStatus::Active);

    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        d(100, 20) / 250 * 249
    );
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.alice),
        initial_balance - d(1, 24)
    );
    assert_eq!(stream.balance, d(1, 24) - d(1, 23) - d(100, 20));

    let stats = e.get_stats();
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(
        t.total_commission_collected,
        d(1, 23) + (d(1, 24) - d(1, 23)) / 250
    );

    e.skip_time(10000);
    e.withdraw(&users.charlie, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(
        stream.status,
        StreamStatus::Finished {
            reason: StreamFinishReason::FinishedNaturally
        }
    );

    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        (d(1, 24) - d(1, 23)) / 250 * 249
    );
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.alice),
        initial_balance - d(1, 24)
    );
    assert_eq!(stream.balance, 0);

    let stats = e.get_stats();
    let t = stats.dao_tokens.get(&tokens.wnear.account_id()).unwrap();
    assert_eq!(
        t.total_commission_collected,
        d(1, 23) + (d(1, 24) - d(1, 23)) / 250
    );
}

#[test]
fn test_nft_sanity() {
    let (e, tokens, users, nfts) = basic_nft_setup();

    let amount = d(101, 23);
    let stream_id = e.create_stream(
        &users.alice,
        &users.charlie,
        &tokens.wnear,
        amount,
        d(1, 23),
    );

    let nft_id = "123".to_string();

    e.nft_mint(&nfts.paras, &users.charlie, &nft_id);
    e.nft_attach_stream(&nfts.paras, &nft_id, &stream_id);
    assert_eq!(e.get_balance(&tokens.wnear, &users.charlie), 0);

    let nft = e.get_nft_token(&nfts.paras, &nft_id).unwrap();
    assert_eq!(
        nft.metadata.unwrap().extra.unwrap(),
        String::from(&stream_id)
    );

    e.skip_time(10);

    e.nft_transfer(&users.charlie, &nfts.paras, &users.dude, &nft_id);
    let nft = e.get_nft_token(&nfts.paras, &nft_id).unwrap();
    assert_eq!(
        nft.metadata.unwrap().extra.unwrap(),
        String::from(&stream_id)
    );

    // TODO enable #11
    /*let stream = e.get_stream(&stream_id);
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        d(10, 23) / 250 * 249
    );
    assert_eq!(e.get_balance(&tokens.wnear, &users.dude), 0);
    assert_eq!(stream.balance, d(90, 23) - d(1, 23));

    e.skip_time(20);

    assert!(!e.withdraw_err(&users.charlie, &stream_id).is_ok());
    e.withdraw(&users.dude, &stream_id);
    let stream = e.get_stream(&stream_id);
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.charlie),
        d(10, 23) / 250 * 249
    );
    assert_eq!(
        e.get_balance(&tokens.wnear, &users.dude),
        d(20, 23) / 250 * 249
    );
    assert_eq!(stream.balance, d(70, 23) - d(1, 23));*/

    e.nft_detach_stream(&nfts.paras, &nft_id, &stream_id);

    let nft = e.get_nft_token(&nfts.paras, &nft_id).unwrap();
    assert!(nft.metadata.unwrap().extra.is_none());
}
