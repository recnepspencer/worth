use worth_runtime_bridge::facade::BridgeAsyncRetryLineageRequest;

fn main() {
    let _ = BridgeAsyncRetryLineageRequest {
        prior: supplied(),
        timeout_report: None,
        cancellation_report: None,
        retry_schedule_report: supplied(),
        retry_admission_report: supplied(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
