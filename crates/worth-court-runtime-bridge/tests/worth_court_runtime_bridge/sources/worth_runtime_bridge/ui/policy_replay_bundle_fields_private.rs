use worth_runtime_bridge::facade::BridgePolicyReplayBundle;

fn main() {
    let _bundle = BridgePolicyReplayBundle {
        contract_digest: sealed_authority_placeholder(),
        lowered_policy_digest: sealed_authority_placeholder(),
        provenance_digest: sealed_authority_placeholder(),
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
