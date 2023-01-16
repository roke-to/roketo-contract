use log::info;
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
pub const VAULT_WASM_PATH: &str = "res/nft_benefits_vault.wasm";
pub const NFT_WASM_PATH: &str = "res/non_fungible_token.wasm";

pub const NFT_TOKEN_ID: &str = "42";

pub fn init_logger() {
    if let Err(e) = env_logger::Builder::new()
        .parse_env("RUST_LOG")
        .format_timestamp(None)
        .format_module_path(false)
        .format_target(false)
        .try_init()
    {
        info!("logger already initialized: {}", e);
    }
}

pub fn assert_all_success(res: &ExecutionFinalResult) {
    if res
        .receipt_outcomes()
        .iter()
        .any(|receipt| receipt.is_failure())
    {
        panic!("{res:#?}");
    }
}
