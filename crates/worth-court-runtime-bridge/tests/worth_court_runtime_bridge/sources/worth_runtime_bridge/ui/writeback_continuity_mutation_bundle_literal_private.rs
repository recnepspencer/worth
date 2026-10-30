use worth_runtime_bridge::facade::BridgeContinuityMutationBundle;

fn main() {
    let _ = BridgeContinuityMutationBundle {
        family: sealed_authority_placeholder(),
        outcome_class: sealed_authority_placeholder(),
        prior_authoritative_identity: sealed_authority_placeholder(),
        successor_authoritative_identities: Vec::new(),
        basis_binding_digest: None,
        resolved_target_entity_identity: None,
        target_collection: None,
        lineage_digest: sealed_authority_placeholder(),
        continuity_resolution_digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
