#[cfg(test)]
mod environment;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod setup;

#[cfg(test)]
mod integration;

#[cfg(test)]
mod pair_with_vault;

#[cfg(test)]
mod ws_tests;

#[cfg(test)]
mod ws_setup;

// Precompiled smart contracts locations.
const STREAMING_WASMS_DIR: &str = "../res";
const EXTERNAL_TEST_WASMS_DIR: &str = "../tests/res";
const WRAP_NEAR_WASM: &str = "wrap_near.wasm";
// const NFT_WASM: &str = "non_fungible_token.wasm";
// const FUNGIBLE_TOKEN_WASM: &str = "fungible_token.wasm";

// Constants related to the wrap NEAR FT contract.
const WRAP_NEAR_TESTNET_ACCOUNT_ID: &str = "wrap.testnet";
// const WRAP_NEAR_DEPOSIT_CALL: &str = "near_deposit";

// Constant parameters for Roketo-streaming accounts setup.
const STREAMING_SUBACCOUNT_ID: &str = "streaming";
const STREAMING_WASM: &str = "streaming.wasm";
const FINANCE_SUBACCOUNT_ID: &str = "finance";
const FINANCE_WASM: &str = "finance.wasm";
const UTILITY_TOKEN_SUBACCOUNT_ID: &str = "roke_token";
const UTILITY_TOKEN_WASM: &str = "roke_token.wasm";
const UTILITY_TOKEN_DECIMALS: i32 = 18;
