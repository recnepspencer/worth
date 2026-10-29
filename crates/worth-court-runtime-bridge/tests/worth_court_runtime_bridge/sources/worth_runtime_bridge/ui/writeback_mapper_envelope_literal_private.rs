use worth_runtime_bridge::facade::BridgeWritebackMapperEnvelope;

fn main() {
    let _ = BridgeWritebackMapperEnvelope {
        envelope_identity: sealed_authority_placeholder(),
        contract_digest: sealed_authority_placeholder(),
        family_kind: sealed_authority_placeholder(),
        effect_class: sealed_authority_placeholder(),
        strategy_class: sealed_authority_placeholder(),
        strategy_descriptor_basis: sealed_authority_placeholder(),
        causality_digest: sealed_authority_placeholder(),
        mutation_subject_effect_intent_match: false,
        effect_intent: sealed_authority_placeholder(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
