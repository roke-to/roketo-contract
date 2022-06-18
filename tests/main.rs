mod setup;

use crate::setup::*;

#[tokio::test]
async fn test_hello2() -> anyhow::Result<()> {
    println!("Hello World");
    assert_eq!(0, 2);
    let sandbox = sandbox().await?;
    let wasm = std::fs::read(STREAMING_ID)?;
    let contract = sandbox.dev_deploy(&wasm).await?;

    let owner = sandbox.root_account();
    let user = owner
        .create_subaccount(&sandbox, "user")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    test_finance_transfers().await?;
    Ok(())
}

#[tokio::test]
async fn test_hello() -> anyhow::Result<()> {
    println!("Hello World");
    assert_eq!(0, 1);
    let sandbox = sandbox().await?;
    let wasm = std::fs::read(STREAMING_ID)?;
    let contract = sandbox.dev_deploy(&wasm).await?;

    let owner = sandbox.root_account();
    let user = owner
        .create_subaccount(&sandbox, "user")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    test_finance_transfers().await?;
    Ok(())
}

async fn test_finance_transfers() -> anyhow::Result<()> {
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
    Ok(())
}
