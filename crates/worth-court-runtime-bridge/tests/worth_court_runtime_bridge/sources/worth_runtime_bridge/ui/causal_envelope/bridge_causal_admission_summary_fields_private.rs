use worth_runtime_bridge::facade::{
    BridgeCausalInspectionAdmissionSummary, BridgeCausalInspectionAdmissionSummaryKind,
};

fn main() {
    let _ = BridgeCausalInspectionAdmissionSummary {
        kind: BridgeCausalInspectionAdmissionSummaryKind::Admitted,
        query_admission_digest: sealed_authority_placeholder(),
        causal_observation_anchor_digest: sealed_authority_placeholder(),
        summary_digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
