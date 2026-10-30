#![cfg_attr(test, allow(clippy::disallowed_methods))]
#[expect(
    clippy::disallowed_methods,
    reason = "async-runtime async-runtime=1: This Bank process owns its I/O executor; computation retains installed Execution authority"
)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bank_user_node::run_bank_user_node_process().await
}
