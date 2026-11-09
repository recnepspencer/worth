use worth_runtime_bridge::facade::{
    BridgeAsyncForwardCausalityClass, BridgeAsyncForwardCausalityReceipt, BridgeAsyncRetryLineage,
};

fn main() {
    let _ = BridgeAsyncRetryLineage {
        causality_identity: supplied(),
        prior_request: supplied(),
        newer_request: supplied(),
        class: BridgeAsyncForwardCausalityClass::RetryAfterTimeout,
        counters: supplied(),
        receipt: supplied(),
        canonical_basis: supplied(),
        digest: supplied(),
    };
    let _ = BridgeAsyncForwardCausalityReceipt {
        receipt_identity: supplied(),
        causality_identity: supplied(),
        class: BridgeAsyncForwardCausalityClass::RetryAfterTimeout,
        canonical_basis: supplied(),
        digest: supplied(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
