use worth_runtime_bridge::facade::{
    BridgeBatchMutationAuthorityBundle, BridgeMutationAuthorityBundle,
    BridgeMutationCausalityBundle, BridgeMutationProvenanceBundle,
};

fn main() {
    let causality = BridgeMutationCausalityBundle {
        causality_digest: sealed_authority_placeholder(),
        truth_trigger_digest: sealed_authority_placeholder(),
        route_digest: sealed_authority_placeholder(),
        evaluation_surface_digest: sealed_authority_placeholder(),
        truth_view_digest: sealed_authority_placeholder(),
    };
    let provenance = BridgeMutationProvenanceBundle {
        contract_digest: sealed_authority_placeholder(),
        writeback_effect_artifact_digest: sealed_authority_placeholder(),
        effect_intent_digest: sealed_authority_placeholder(),
        effect_intent_patch_canonical_basis: sealed_authority_placeholder(),
        feedback_provenance_digest: sealed_authority_placeholder(),
        causality_digest: sealed_authority_placeholder(),
        strategy_descriptor_basis: sealed_authority_placeholder(),
        execution_record_digest: sealed_authority_placeholder(),
        outcome_class: None,
        authoritative_artifact_digest: None,
        authority_request: None,
        authority_receipt: None,
        failure_class: None,
    };
    let _ = BridgeMutationAuthorityBundle {
        causality,
        provenance,
    };
    let _ = BridgeBatchMutationAuthorityBundle {
        component_count: 0,
        causality_bundle_count: 0,
        provenance_bundle_count: 0,
        outcome_class_count: 0,
        authority_request_count: 0,
        authority_receipt_count: 0,
        aggregate_causality_digest: sealed_authority_placeholder(),
        aggregate_provenance_digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
