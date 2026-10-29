pub(crate) use worth_query_bank_testkit_server::facade::estate_operations::phase8_publication_noninterference::*;

#[test]
fn paired_protected_account_detail_worlds_publish_equal_complete_commit_surfaces() {
    let left = freeze_world_with_protected_foreign_status(
        "r848-protected-twin",
        AccountStatus::Open,
        AccountStatus::Open,
    );
    let right = freeze_world_with_protected_foreign_status(
        "r848-protected-twin",
        AccountStatus::Open,
        AccountStatus::Closed,
    );
    assert_eq!(protected_foreign_status(&left), AccountStatus::Open);
    assert_eq!(protected_foreign_status(&right), AccountStatus::Closed);

    let left = commit_freeze(&left, 31);
    let right = commit_freeze(&right, 31);

    // Each receipt carries its real World publication identity. Compare the
    // Bank-visible semantics rather than pretending two Worlds share authority.
    assert_eq!(
        left.publication().inspect().kind(),
        right.publication().inspect().kind()
    );
    assert_eq!(left.changed_record_count(), right.changed_record_count());
    assert_eq!(left.emitted_effect_count(), right.emitted_effect_count());
    assert_eq!(
        left.expected_version_count(),
        right.expected_version_count()
    );
    assert_eq!(left.expected_fact_count(), right.expected_fact_count());
    assert_eq!(left.decision_fact_count(), right.decision_fact_count());
    assert_eq!(left.canonical_work(), right.canonical_work());
    assert_eq!(left.aftermath(), right.aftermath());
    assert_eq!(
        left.external_dispatch_posture(),
        right.external_dispatch_posture()
    );
    assert_eq!(
        left.co_committed_dispatch_outbox(),
        right.co_committed_dispatch_outbox()
    );
    assert_eq!(left.retained_preimage(), right.retained_preimage());
    assert_eq!(
        left.performed_preimage_retention_work(),
        right.performed_preimage_retention_work()
    );
    assert_eq!(
        left.aftermath().posture(),
        Some(WorthQueryPublishedAftermathPosture::Reversible)
    );
}
