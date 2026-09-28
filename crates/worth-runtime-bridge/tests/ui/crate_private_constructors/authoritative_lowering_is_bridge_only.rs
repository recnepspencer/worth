use worth_runtime_bridge::facade::{
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchItem, BridgeProducerAuthorityKind,
    BridgeProducerMetadata,
};

fn main() {
    let _envelope = BridgeCommittedPatchEnvelope::new_with_authoritative_lowering(
        sealed_authority_placeholder(),
        Vec::new(),
        Vec::new(),
        sealed_authority_placeholder(),
    );
    let _item = BridgeCommittedPatchItem::with_relational_semantic_change(
        sealed_authority_placeholder(),
        sealed_authority_placeholder(),
        sealed_authority_placeholder(),
    );
    let _claimed = BridgeProducerMetadata::new(
        BridgeProducerAuthorityKind::RegisteredAuthoritativeSource,
        "worth-runtime-bridge.producer-envelope.v1",
    );
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
