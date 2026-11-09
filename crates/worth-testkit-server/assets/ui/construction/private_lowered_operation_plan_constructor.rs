use worth_server::WorthServerLoweredOperationPlan;

fn main() {
    let _ = WorthServerLoweredOperationPlan {
        query_handoff: supplied(),
        strategy: supplied(),
        evidence_policy: supplied(),
        counters: supplied(),
        receipt: supplied(),
        canonical_digest: "Worthd".to_string(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
