use super::MilestoneFiveLiveCertificationAdapter;

#[test]
fn live_query_patch_policy_artifact_is_offline_ready() {
    let artifact =
        MilestoneFiveLiveCertificationAdapter::
            live_promotion_convergence_and_suppression_certification_artifact();

    assert!(artifact.counter_snapshot.live_patch_delivery_count() > 0);
    assert!(artifact.counter_snapshot.live_invalidation_event_count() > 0);
    assert!(artifact.counter_snapshot.live_relevance_match_count() > 0);
    assert!(
        artifact
            .counter_snapshot
            .live_irrelevant_suppression_count()
            > 0
    );
    assert!(artifact.counter_snapshot.live_patch_count() > 0);
    assert!(artifact.counter_snapshot.live_patch_field_delta_count() > 0);
    assert!(artifact.counter_snapshot.live_collection_reorder_count() > 0);
    assert!(artifact.counter_snapshot.live_materialization_patch_count() > 0);
    assert!(artifact.counter_snapshot.live_refresh_fallback_count() > 0);
    assert!(artifact.counter_snapshot.live_replay_change_count() > 0);
    assert!(
        artifact
            .counter_snapshot
            .live_coalesced_change_bundle_count()
            > 0
    );
    assert!(artifact.counter_snapshot.live_patch_width_overflow_count() > 0);
    assert!(artifact.counter_snapshot.live_refresh_cost_class_count() > 0);
    assert!(
        artifact
            .counter_snapshot
            .live_work_avoided_by_irrelevance_count()
            > 0
    );
    assert!(
        artifact
            .counter_snapshot
            .live_work_avoided_by_stable_ordering_count()
            > 0
    );
    assert!(
        artifact
            .counter_snapshot
            .live_work_avoided_by_scope_proof_count()
            > 0
    );
    assert_eq!(
        artifact.counter_snapshot.live_executor_rediscovery_count(),
        0
    );
    assert!(artifact.counter_snapshot.live_progress_advance_count() > 0);
    assert!(
        artifact
            .counter_snapshot
            .live_non_monotonic_sequence_rejection_count()
            > 0
    );
    assert!(artifact.counter_snapshot.live_change_sequence_gap_count() > 0);
    assert!(
        artifact
            .counter_snapshot
            .live_invalid_promotion_rejection_count()
            > 0
    );
    assert!(
        artifact
            .counter_snapshot
            .live_unsupported_patch_family_rejection_count()
            > 0
    );
    assert!(artifact
        .matrix
        .rows
        .iter()
        .all(|row| row.has_required_outputs()));
    let uncovered_rows: Vec<_> = artifact
        .matrix
        .rows
        .iter()
        .filter(|row| !row.has_hostile_coverage())
        .map(|row| row.row_name)
        .collect();
    assert!(
        uncovered_rows.is_empty(),
        "rows without hostile coverage: {uncovered_rows:?}"
    );
    assert!(artifact
        .matrix
        .rejection_rows
        .iter()
        .all(|row| row.has_required_outputs()));
    assert!(artifact
        .matrix
        .rejection_rows
        .iter()
        .all(|row| row.has_hostile_coverage()));
}

#[test]
fn live_query_patch_policy_artifact_is_deterministic() {
    let left =
        MilestoneFiveLiveCertificationAdapter::
            live_promotion_convergence_and_suppression_certification_artifact();
    let right =
        MilestoneFiveLiveCertificationAdapter::
            live_promotion_convergence_and_suppression_certification_artifact();

    assert_eq!(
        left.certification_bundle_digest,
        right.certification_bundle_digest
    );
    assert_eq!(left.coverage_matrix_digest, right.coverage_matrix_digest);
    assert_eq!(left.counter_snapshot, right.counter_snapshot);
    assert_eq!(left.matrix, right.matrix);
}
