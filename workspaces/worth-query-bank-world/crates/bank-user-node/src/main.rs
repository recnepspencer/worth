#![cfg_attr(test, allow(clippy::disallowed_methods))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bank_user_node::run_bank_user_node_process().await
}
