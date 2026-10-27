use worth_signal::facade::branch::{SignalConditionalExecutionPort, SignalOwnedAsyncSourceBinding};
use worth_signal::facade::{ResourceRequestHandle, ResourceRevalidationIntent};

fn source_and_handle_are_not_cleanup_custody(
    port: &SignalConditionalExecutionPort<u32, u32, u32>,
    source: &SignalOwnedAsyncSourceBinding,
    handle: ResourceRequestHandle,
    intent: ResourceRevalidationIntent,
) {
    let _ = port.admit_owned_async_request(source);
    let _ = port.advance_owned_async_request_to_timeout(source, handle, 5);
    let _ = port.schedule_owned_async_retry(source, handle);
    let _ = port.revalidate_owned_async_request(source, intent);
}

fn main() {}
