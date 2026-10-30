use worth_runtime_bridge::facade::BridgeNamingMutationBundle;

fn main() {
    let _ = BridgeNamingMutationBundle {
        family: sealed_authority_placeholder(),
        outcome: sealed_authority_placeholder(),
        attachment_identity: sealed_authority_placeholder(),
        prior_authoritative_identity: None,
        target_authoritative_identity: None,
        resolved_target_entity_identity: None,
        target_collection: None,
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
