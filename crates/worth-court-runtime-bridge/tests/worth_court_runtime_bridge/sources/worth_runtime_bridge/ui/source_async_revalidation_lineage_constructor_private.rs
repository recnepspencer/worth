use worth_runtime_bridge::facade::{
    BridgeAsyncForwardCausalityClass, BridgeAsyncRevalidationLineage,
};

fn main() {
    let _ = BridgeAsyncRevalidationLineage {
        causality_identity: supplied(),
        prior_request: supplied(),
        newer_request: supplied(),
        class: BridgeAsyncForwardCausalityClass::RevalidationAfterTruthBasisDrift,
        counters: supplied(),
        receipt: supplied(),
        canonical_basis: supplied(),
        digest: supplied(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
