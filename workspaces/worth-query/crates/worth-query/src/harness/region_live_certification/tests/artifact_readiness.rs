use super::super::MilestoneFivePointOneLiveCertificationAdapter;

#[test]
fn region_live_artifact_is_offline_ready() {
    let artifact = MilestoneFivePointOneLiveCertificationAdapter::
        region_scoped_live_narrowing_and_stream_contract_certification_artifact();

    assert!(artifact.counter_snapshot.locality_region_match_count() > 0);
    assert!(
        artifact
            .counter_snapshot
            .locality_irrelevant_broad_control_count()
            > 0
    );
    assert!(artifact.counter_snapshot.locality_replay_change_count() > 0);
    assert!(
        artifact
            .counter_snapshot
            .locality_off_region_suppression_count()
            > 0
    );
    assert!(
        artifact
            .counter_snapshot
            .locality_breadth_budget_cross_count()
            > 0
    );
    assert!(artifact.counter_snapshot.locality_widening_denial_count() > 0);
    assert!(
        artifact
            .counter_snapshot
            .locality_widening_admission_count()
            > 0
    );
    assert!(
        artifact
            .counter_snapshot
            .locality_widening_budget_cross_count()
            > 0
    );
    assert!(
        artifact
            .counter_snapshot
            .locality_bridge_slice_incompatibility_count()
            > 0
    );
    assert!(artifact.counter_snapshot.stream_contract_admission_count() > 0);
    assert!(artifact.counter_snapshot.stream_contract_denial_count() > 0);
    assert!(artifact.counter_snapshot.stream_lowered_delivery_count() > 0);
    assert!(artifact.counter_snapshot.stream_lowered_delivery_width() > 0);
    assert!(
        artifact
            .counter_snapshot
            .stream_window_width_budget_cross_count()
            > 0
    );
    assert!(
        artifact
            .counter_snapshot
            .stream_member_width_budget_cross_count()
            > 0
    );
    assert!(
        artifact
            .counter_snapshot
            .locality_work_avoided_by_region_narrowing_count()
            > 0
    );
    assert!(
        artifact
            .counter_snapshot
            .locality_work_avoided_vs_broad_control_count()
            > 0
    );
    assert!(
        artifact
            .counter_snapshot
            .locality_unsupported_predicate_rejection_count()
            > 0
    );
    assert_eq!(
        artifact.counter_snapshot.locality_replay_divergence_count(),
        0
    );
    assert_eq!(
        artifact
            .counter_snapshot
            .locality_executor_rediscovery_count(),
        0
    );
    assert!(artifact
        .matrix
        .rows
        .iter()
        .all(|row| row.has_required_outputs()));
    assert!(artifact
        .matrix
        .rejection_rows
        .iter()
        .all(|row| row.has_required_outputs()));
}
