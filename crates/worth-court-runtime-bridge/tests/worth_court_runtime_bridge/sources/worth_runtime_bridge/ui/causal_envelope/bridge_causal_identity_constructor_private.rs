use worth_runtime_bridge::facade::BridgeCausalEnvelopeIdentity;

fn main() {
    let _ = BridgeCausalEnvelopeIdentity {
        request_digest: sealed_authority_placeholder(),
        causal_observation_anchor_digest: sealed_authority_placeholder(),
        evidence_binding_digest: sealed_authority_placeholder(),
        counter_digest: sealed_authority_placeholder(),
        identity_digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
