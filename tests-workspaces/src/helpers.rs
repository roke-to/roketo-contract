use log::info;
use workspaces::result::ExecutionFinalResult;

pub fn assert_all_success(res: &ExecutionFinalResult) {
    if res
        .receipt_outcomes()
        .iter()
        .any(|receipt| receipt.is_failure())
    {
        panic!("{res:#?}");
    }
}

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
