mod setup;

use crate::setup::*;

#[tokio::test]
async fn test_finance_transfers() -> anyhow::Result<()> {
    let (e, tokens, users) = basic_setup().await?;

    //    assert_eq!(e.get_balance(&tokens.wnear_simple, &e.streaming).await?, 0);
    // assert_eq!(e.get_balance(&tokens.wnear_simple, &e.finance).await?, 0);

    // let near_streaming = e.get_near_balance(&e.streaming).await?;
    // let near_finance = e.get_near_balance(&e.finance).await?;

    // let amount = d(101, 23);
    // e.create_stream_ext_err(
    //     &users.alice,
    //     &users.charlie,
    //     &tokens.wnear_simple,
    //     amount,
    //     d(1, 23),
    //     None,
    //     None,
    //     None,
    //     None,
    //     None,
    // );

    // assert_eq!(
    //     e.get_balance(&tokens.wnear_simple, &e.streaming).await?,
    //     d(1, 23)
    // );
    // assert_eq!(
    //     e.get_balance(&tokens.wnear_simple, &e.finance).await?,
    //     d(100, 23)
    // );

    // assert!(
    //     e.get_near_balance(&e.streaming).await?
    //         > near_streaming + STORAGE_NEEDS_PER_STREAM - STORAGE_NEEDS_PER_STREAM / 100
    // );
    // assert!(
    //     e.get_near_balance(&e.streaming).await?
    //         < near_streaming + STORAGE_NEEDS_PER_STREAM + STORAGE_NEEDS_PER_STREAM / 100
    // );
    // assert!(
    //     e.get_near_balance(&e.finance).await?
    //         > near_finance - STORAGE_NEEDS_PER_STREAM - STORAGE_NEEDS_PER_STREAM / 100
    // );
    // assert!(
    //     e.get_near_balance(&e.finance).await?
    //         < near_finance - STORAGE_NEEDS_PER_STREAM + STORAGE_NEEDS_PER_STREAM / 100
    // );
    Ok(())
}
