use worth_signal::facade::HostComputedPreparedResponse;

fn main() {
    let _ = HostComputedPreparedResponse {
        prepared: supplied(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
