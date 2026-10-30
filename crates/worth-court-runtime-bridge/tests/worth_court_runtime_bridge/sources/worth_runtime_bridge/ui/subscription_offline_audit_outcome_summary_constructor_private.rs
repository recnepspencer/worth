use worth_runtime_bridge::facade::BridgeSubscriptionOfflineAuditOutcomeSummary;

fn main() {
    let _summary = BridgeSubscriptionOfflineAuditOutcomeSummary {
        equivalent_count: 0,
        intentionally_divergent_count: 0,
        expected_rejection_count: 0,
        unexpected_rejection_count: 0,
        diagnostics_only_count: 0,
        residue_mismatch_count: 0,
        replay_mismatch_count: 0,
        counter_contract_violation_count: 0,
        bundle_completeness_violation_count: 0,
        canonical_basis: sealed_authority_placeholder(),
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
