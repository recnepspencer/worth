use worth_runtime_bridge::facade::BridgeCausalEnvelopeReceipt;

fn main() {
    let _ = BridgeCausalEnvelopeReceipt {
        envelope_identity_digest: sealed_authority_placeholder(),
        envelope_digest: sealed_authority_placeholder(),
        counter_digest: sealed_authority_placeholder(),
        receipt_digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
