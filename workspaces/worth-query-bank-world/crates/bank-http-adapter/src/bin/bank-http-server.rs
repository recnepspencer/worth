#![cfg_attr(test, allow(clippy::disallowed_methods))]
#[expect(
    clippy::disallowed_methods,
    reason = "async-runtime async-runtime=1: Bank HTTP ingress owns this I/O executor; computation retains its installed Execution authority"
)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    worth_execution::run_with_execution_stack(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(bank_http_adapter::run_bank_http_server_process())
    })
}
