use workspaces::result::ExecutionFinalResult;

#[cfg(test)]
mod args;
#[cfg(test)]
mod environment;
#[cfg(test)]
mod tests;

pub const STREAMING_WASM_PATH: &str = "../res/streaming.wasm";
pub const FINANCE_WASM_PATH: &str = "../res/finance.wasm";
pub const WRAP_NEAR_WASM_PATH: &str = "../tests-integration/res/wrap_near.wasm";
pub const VAULT_WASM_PATH: &str = "../res/nft_benefits_vault.wasm";

pub fn assert_all_success(res: &ExecutionFinalResult) {
    if res
        .receipt_outcomes()
        .iter()
        .any(|receipt| receipt.is_failure())
    {
        panic!("{res:#?}");
    }
}
