use near_sdk::{
    json_types::U128,
    serde_json::{json, Value},
};
use workspaces::AccountId;

use crate::NFT_TOKEN_ID;

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

pub fn wrap_near_ft_transfer_call_json(
    to: &AccountId,
    amount: u128,
    owner_id: &AccountId,
    vault_id: &AccountId,
    nft_contract_id: &AccountId,
) -> Value {
    let amount = U128(amount);
    let vault_request = json!({
        "nft_id": NFT_TOKEN_ID,
        "nft_contract_id": nft_contract_id,
        "kind": "Transfer",
    })
    .to_string();
    let withdraw_args = json!({
        "stream_id": "",
        "msg": vault_request,
    })
    .to_string();
    let vault_args = json!({
        "nft_contract_id": nft_contract_id,
        "nft_id": NFT_TOKEN_ID,
        "callback": "withdraw_call",
        "args": withdraw_args,
        "deposit": U128(1),
    })
    .to_string();
    let request = json!({
        "owner_id": owner_id,
        "receiver_id": vault_id,
        "tokens_per_sec": amount,
        "is_auto_start_enabled": true,
    });
    let msg = json!({
        "CreateCall": {
            "request": request,
            "contract": vault_id,
            "call": "add_replenishment_callback",
            "args": vault_args,
        },
    })
    .to_string();
    json!({
        "receiver_id": to,
        "amount": amount,
        "msg": msg,
    })
}
