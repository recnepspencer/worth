use worth_runtime_bridge::facade::{
    BridgeAsyncForwardCausalityClass, BridgeAsyncForwardCausalityReceipt,
    BridgeAsyncRetryLineage,
};

fn main() {
    let _ = BridgeAsyncRetryLineage {
        causality_identity: todo!(),
        prior_request: todo!(),
        newer_request: todo!(),
        class: BridgeAsyncForwardCausalityClass::RetryAfterTimeout,
        counters: todo!(),
        receipt: todo!(),
        canonical_basis: todo!(),
        digest: todo!(),
    };
    let _ = BridgeAsyncForwardCausalityReceipt {
        receipt_identity: todo!(),
        causality_identity: todo!(),
        class: BridgeAsyncForwardCausalityClass::RetryAfterTimeout,
        canonical_basis: todo!(),
        digest: todo!(),
    };
}
