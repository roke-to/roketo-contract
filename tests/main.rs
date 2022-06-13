mod setup;

use crate::setup::*;

#[test]
fn test_finance_transfers() {
    let (e, tokens, users) = basic_setup();

    assert_eq!(
        e.get_balance(&tokens.wnear_simple, &e.streaming.user_account),
        0
    );
    assert_eq!(e.get_balance(&tokens.wnear_simple, &e.finance), 0);

    let near_streaming = e.get_near_balance(&e.streaming.user_account);
    let near_finance = e.get_near_balance(&e.finance);

    let amount = d(101, 23);
    e.create_stream_ext_err(
        &users.alice,
        &users.charlie,
        &tokens.wnear_simple,
        amount,
        d(1, 23),
        None,
        None,
        None,
        None,
        None,
    );

    assert_eq!(
        e.get_balance(&tokens.wnear_simple, &e.streaming.user_account),
        d(1, 23)
    );
    assert_eq!(e.get_balance(&tokens.wnear_simple, &e.finance), d(100, 23));

    assert!(
        e.get_near_balance(&e.streaming.user_account)
            > near_streaming + STORAGE_NEEDS_PER_STREAM - STORAGE_NEEDS_PER_STREAM / 100
    );
    assert!(
        e.get_near_balance(&e.streaming.user_account)
            < near_streaming + STORAGE_NEEDS_PER_STREAM + STORAGE_NEEDS_PER_STREAM / 100
    );
    assert!(
        e.get_near_balance(&e.finance)
            > near_finance - STORAGE_NEEDS_PER_STREAM - STORAGE_NEEDS_PER_STREAM / 100
    );
    assert!(
        e.get_near_balance(&e.finance)
            < near_finance - STORAGE_NEEDS_PER_STREAM + STORAGE_NEEDS_PER_STREAM / 100
    );
}
