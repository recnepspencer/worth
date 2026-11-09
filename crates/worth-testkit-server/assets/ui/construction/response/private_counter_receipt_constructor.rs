use worth_server::WorthServerOperatorCounterReceipt;

fn main() {
    let _ = WorthServerOperatorCounterReceipt { inner: supplied() };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
