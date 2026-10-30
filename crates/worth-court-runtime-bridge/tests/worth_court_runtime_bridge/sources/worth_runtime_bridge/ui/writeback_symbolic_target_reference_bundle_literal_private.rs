use worth_runtime_bridge::facade::BridgeSymbolicTargetReferenceBundle;

fn main() {
    let _ = BridgeSymbolicTargetReferenceBundle {
        family: sealed_authority_placeholder(),
        outcome: sealed_authority_placeholder(),
        symbol: sealed_authority_placeholder(),
        resolved_entity_identity: sealed_authority_placeholder(),
        target_collection: None,
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
