use worth_runtime_bridge::facade::{
    BridgeCausalEnvelopeCounters, BridgeCausalEnvelopeIdentity, BridgeCausalEnvelopeReceipt,
    BridgeCausalEvidenceBinding, BridgeCausalExplanationEnvelope,
    BridgeCausalInspectionAdmissionSummaryKind,
};

fn counters() -> BridgeCausalEnvelopeCounters {
    sealed_authority_placeholder()
}

fn identity() -> BridgeCausalEnvelopeIdentity {
    sealed_authority_placeholder()
}

fn receipt() -> BridgeCausalEnvelopeReceipt {
    sealed_authority_placeholder()
}

fn main() {
    let bindings: Vec<BridgeCausalEvidenceBinding> = Vec::new();
    let counters = counters();

    let _ = BridgeCausalExplanationEnvelope {
        identity: identity(),
        admission_summary_kind: BridgeCausalInspectionAdmissionSummaryKind::Admitted,
        admission_summary_digest: sealed_authority_placeholder(),
        request_digest: sealed_authority_placeholder(),
        causal_observation_anchor_digest: sealed_authority_placeholder(),
        bindings: bindings.into(),
        counters,
        receipt: receipt(),
        envelope_digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
