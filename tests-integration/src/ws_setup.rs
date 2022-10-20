pub use near_sdk::serde_json::json;
pub use near_sdk::ONE_YOCTO;
pub use near_units::parse_near;
pub use workspaces::{Account, Contract, Worker};
pub use workspaces::DevNetwork;

pub use streaming::Dao;

pub async fn init(
    worker: &Worker<impl DevNetwork>,
    // worker: &Worker<workspaces::network::Sandbox>,
    dao_account: &Account,
) -> anyhow::Result<(Contract, Contract, Contract, Contract)> {
    // let master_account = worker.dev_create_account().await?;

    // deployment of contracts
    // deploying under Dao-subaccounts, to obtain better namings.

    // let streaming_contract = worker
    //     .dev_deploy(include_bytes!("../../res/streaming.wasm").as_ref())
    //     .await?;

    let streaming_account = dao_account
        .create_subaccount("streaming")
        .initial_balance(parse_near!("10.00 N"))
        .transact()
        .await?
        .into_result()?;

    let streaming_contract = streaming_account
        .deploy(include_bytes!("../../res/streaming.wasm").as_ref())
        .await?
        .into_result()?;

    // let finance_contract = worker
    //     .dev_deploy(include_bytes!("../../res/finance.wasm").as_ref())
    //     .await?;

    let finance_account = dao_account
        .create_subaccount("finance")
        .initial_balance(parse_near!("10.00 N"))
        .transact()
        .await?
        .into_result()?;

    let finance_contract = finance_account
        .deploy(include_bytes!("../../res/finance.wasm").as_ref())
        .await?
        .into_result()?;

    // let token_contract = worker
    //     .dev_deploy(include_bytes!("../../res/roke_token.wasm").as_ref())
    //     .await?;

    let token_account = dao_account
        .create_subaccount("token")
        .initial_balance(parse_near!("10.00 N"))
        .transact()
        .await?
        .into_result()?;

    let token_contract = token_account
        .deploy(include_bytes!("../../res/roke_token.wasm").as_ref())
        .await?
        .into_result()?;

    // let wrap_contract = worker
    //     .dev_deploy(include_bytes!("../res/wrap_near.wasm").as_ref())
    //     .await?;

    let wrap_account = dao_account
        .create_subaccount("wrap")
        .initial_balance(parse_near!("10.00 N"))
        .transact()
        .await?
        .into_result()?;

    let wrap_contract = wrap_account
        .deploy(include_bytes!("../res/wrap_near.wasm").as_ref())
        .await?
        .into_result()?;

    // init wrap contract
    let res = dao_account
        .call(wrap_contract.id(), "new")
        .transact()
        .await?;
    assert!(res.is_success());

    // init finance and streaming contracts
    let res = dao_account
        .call(finance_contract.id(), "new")
        .args_json((streaming_contract.as_account().id(),))
        .transact()
        .await?;
    assert!(res.is_success());

    let res = dao_account
        .call(streaming_contract.id(), "new")
        .args_json((
            dao_account.id(),
            finance_contract.as_account().id(),
            token_contract.as_account().id(),
            18,
        ))
        .transact()
        .await?;
    assert!(res.is_success());

    // register roketo FT
    let res = dao_account
        .call(token_contract.id(), "new")
        .args_json(json!({
            "owner_id": &dao_account.id(),
            "total_supply": "10000000000000000000",
            "metadata":  {
                "spec": "ft-1.0.0",
                "name": "Test Token",
                "symbol": "TST",
                "decimals": 18 }
        }))
        .transact()
        .await?;
    assert!(res.is_success());

    // adding ft to streaming
    // add wrap token
    let res = dao_account
        .call(streaming_contract.id(), "dao_update_token")
        .args_json(json!({
            "token": {
                "account_id": wrap_contract.id(),
                "is_payment": true,
                "commission_on_transfer": "0",
                "commission_on_create": "10000",
                "commission_coef": { "val": 4, "pow": -3 },
                "collected_commission": "0",
                "storage_balance_needed": "0",
                "gas_for_ft_transfer": "200000000000000",
                "gas_for_storage_deposit": "200000000000000"
            }
        }))
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    // add util token
    let res = dao_account
        .call(streaming_contract.id(), "dao_update_token")
        .args_json(json!({
            "token": {
                "account_id": token_contract.id(),
                "is_payment": false,
                "commission_on_transfer": "0",
                "commission_on_create": "10000",
                "commission_coef": { "val": 1, "pow": -2 },
                "collected_commission": "0",
                "storage_balance_needed": "0",
                "gas_for_ft_transfer": "200000000000000",
                "gas_for_storage_deposit": "200000000000000",
            }
        }))
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    // register contract accounts
    // storage deposit in each FT for finance and streaming accounts
    let res = dao_account
        .call(wrap_contract.id(), "storage_deposit")
        .args_json(json!({
            "account_id": finance_contract.id()
        }))
        .deposit(parse_near!("0.0125 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    let res = dao_account
        .call(wrap_contract.id(), "storage_deposit")
        .args_json(json!({
            "account_id": streaming_contract.id()
        }))
        .deposit(parse_near!("0.0125 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    // storage deposit for Roketo FT
    let res = dao_account
        .call(token_contract.id(), "storage_deposit")
        .args_json(json!({
            "account_id": finance_contract.id()
        }))
        .deposit(parse_near!("0.0125 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    let res = dao_account
        .call(token_contract.id(), "storage_deposit")
        .args_json(json!({
            "account_id": streaming_contract.id()
        }))
        .deposit(parse_near!("0.0125 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    Ok((
        streaming_contract,
        finance_contract,
        token_contract,
        wrap_contract,
    ))
}

pub async fn init_no_wrap_list(
    worker: &Worker<impl DevNetwork>,
    // worker: &Worker<workspaces::network::Sandbox>,
    dao_account: &Account,
) -> anyhow::Result<(Contract, Contract, Contract, Contract)> {
    // let master_account = worker.dev_create_account().await?;

    // deployment of contracts
    let streaming_contract = worker
        .dev_deploy(include_bytes!("../../res/streaming.wasm").as_ref())
        .await?;

    let finance_contract = worker
        .dev_deploy(include_bytes!("../../res/finance.wasm").as_ref())
        .await?;

    let token_contract = worker
        .dev_deploy(include_bytes!("../../res/roke_token.wasm").as_ref())
        .await?;

    let wrap_contract = worker
        .dev_deploy(include_bytes!("../res/wrap_near.wasm").as_ref())
        .await?;

    // init wrap contract
    let res = dao_account
        .call(wrap_contract.id(), "new")
        .transact()
        .await?;
    assert!(res.is_success());

    // init finance and streaming contracts
    let res = dao_account
        .call(finance_contract.id(), "new")
        .args_json((streaming_contract.as_account().id(),))
        .transact()
        .await?;
    assert!(res.is_success());

    let res = dao_account
        .call(streaming_contract.id(), "new")
        .args_json((
            dao_account.id(),
            finance_contract.as_account().id(),
            token_contract.as_account().id(),
            18,
        ))
        .transact()
        .await?;
    assert!(res.is_success());

    // register roketo FT
    let res = dao_account
        .call(token_contract.id(), "new")
        .args_json(json!({
            "owner_id": &dao_account.id(),
            "total_supply": "10000000000000000000",
            "metadata":  {
                "spec": "ft-1.0.0",
                "name": "Test Token",
                "symbol": "TST",
                "decimals": 18 }
        }))
        .transact()
        .await?;
    assert!(res.is_success());

    // adding ft to streaming
    // add wrap token - NO actually don't
    // let res = dao_account
    //     .call(streaming_contract.id(), "dao_update_token")
    //     .args_json(json!({
    //         "token": {
    //             "account_id": wrap_contract.id(),
    //             "is_payment": true,
    //             "commission_on_transfer": "0",
    //             "commission_on_create": "10000",
    //             "commission_coef": { "val": 4, "pow": -3 },
    //             "collected_commission": "0",
    //             "storage_balance_needed": "0",
    //             "gas_for_ft_transfer": "200000000000000",
    //             "gas_for_storage_deposit": "200000000000000"
    //         }
    //     }))
    //     .deposit(ONE_YOCTO)
    //     .transact()
    //     .await?;
    // assert!(res.is_success());

    // add util token
    let res = dao_account
        .call(streaming_contract.id(), "dao_update_token")
        .args_json(json!({
            "token": {
                "account_id": token_contract.id(),
                "is_payment": false,
                "commission_on_transfer": "0",
                "commission_on_create": "10000",
                "commission_coef": { "val": 1, "pow": -2 },
                "collected_commission": "0",
                "storage_balance_needed": "0",
                "gas_for_ft_transfer": "200000000000000",
                "gas_for_storage_deposit": "200000000000000",
            }
        }))
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    // register contract accounts
    // storage deposit in each FT for finance and streaming accounts
    let res = dao_account
        .call(wrap_contract.id(), "storage_deposit")
        .args_json(json!({
            "account_id": finance_contract.id()
        }))
        .deposit(parse_near!("0.0125 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    let res = dao_account
        .call(wrap_contract.id(), "storage_deposit")
        .args_json(json!({
            "account_id": streaming_contract.id()
        }))
        .deposit(parse_near!("0.0125 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    // storage deposit for Roketo FT
    let res = dao_account
        .call(token_contract.id(), "storage_deposit")
        .args_json(json!({
            "account_id": finance_contract.id()
        }))
        .deposit(parse_near!("0.0125 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    let res = dao_account
        .call(token_contract.id(), "storage_deposit")
        .args_json(json!({
            "account_id": streaming_contract.id()
        }))
        .deposit(parse_near!("0.0125 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    Ok((
        streaming_contract,
        finance_contract,
        token_contract,
        wrap_contract,
    ))
}
