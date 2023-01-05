use near_sdk::serde_json::{json, Value};
use workspaces::AccountId;

pub fn streaming_new_json(id: &AccountId, finance: &AccountId) -> Value {
    json!({
        "dao_id": id,
        "finance_id": finance,
        "utility_token_id": "token-roketo.testnet",
        "utility_token_decimals": 18
    })
}

pub fn streaming_dao_update_token_json(token: &AccountId) -> Value {
    json!({
        "token": {
            "account_id": token,
            "is_payment": true,
            "commission_on_transfer": "0",
            "commission_on_create": "10000",
            "commission_coef": { "val": 4, "pow": -3 },
            "collected_commission": "0",
            "storage_balance_needed": "0",
            "gas_for_ft_transfer": "200000000000000",
            "gas_for_storage_deposit": "200000000000000"
        }
    })
}

pub fn finance_new_json(streaming: &AccountId) -> Value {
    json!({
        "streaming_account_id": streaming,
    })
}

pub fn wrap_near_storage_deposit_json(account: &AccountId) -> Value {
    json!({
        "account_id": account,
    })
}
