use worth_runtime_bridge::facade::{
    BridgeRequestKind, BridgeWritebackCausalityBasis, BridgeWritebackCausalityIdentity,
    BridgeWritebackDeclaration, BridgeWritebackDeclarationIdentity, BridgeWritebackEffectClass,
    BridgeWritebackFamilyKind, BridgeWritebackIdempotenceClass, BridgeWritebackRequestMode,
    BridgeWritebackStrategyClass, BridgeWritebackStrategyDescriptorBasis,
};

fn main() {
    let _declaration = BridgeWritebackDeclaration {
        declaration_identity: sealed_authority_placeholder::<BridgeWritebackDeclarationIdentity>(),
        request_kind: sealed_authority_placeholder::<BridgeRequestKind>(),
        request_mode: sealed_authority_placeholder::<BridgeWritebackRequestMode>(),
        family_kind: sealed_authority_placeholder::<Option<BridgeWritebackFamilyKind>>(),
        effect_class: sealed_authority_placeholder::<BridgeWritebackEffectClass>(),
        strategy_class: sealed_authority_placeholder::<Option<BridgeWritebackStrategyClass>>(),
        strategy_descriptor_basis:
            sealed_authority_placeholder::<Option<BridgeWritebackStrategyDescriptorBasis>>(),
        idempotence_class: sealed_authority_placeholder::<BridgeWritebackIdempotenceClass>(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };

    let _causality = BridgeWritebackCausalityBasis {
        causality_identity: sealed_authority_placeholder::<BridgeWritebackCausalityIdentity>(),
        truth_trigger_digest: sealed_authority_placeholder(),
        route_digest: sealed_authority_placeholder(),
        evaluation_surface_digest: sealed_authority_placeholder(),
        truth_view_digest: sealed_authority_placeholder(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
