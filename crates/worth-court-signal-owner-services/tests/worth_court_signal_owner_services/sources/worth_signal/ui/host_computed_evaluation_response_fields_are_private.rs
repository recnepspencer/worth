use worth_signal::facade::HostComputedEvaluationResponse;

fn main() {
    let _ = HostComputedEvaluationResponse { inner: supplied() };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
