use worth_runtime_bridge::facade::{BridgeWritebackOutcomeClass, TruthWritebackReceipt};


fn main() {
    let _receipt = TruthWritebackReceipt {
        outcome_class: BridgeWritebackOutcomeClass::AuthoritativeCommit,
        failure_class: None,
        authoritative_artifact_digest: sealed_authority_placeholder(),
        request_digest: sealed_authority_placeholder(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
