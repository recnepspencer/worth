use worth_server::WorthServerLoweredOperationPlan;

fn main() {
    let _ = WorthServerLoweredOperationPlan {
        query_handoff: loop {},
        strategy: loop {},
        evidence_policy: loop {},
        counters: loop {},
        receipt: loop {},
        canonical_digest: "Worthd".to_string(),
    };
}
