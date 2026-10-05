use super::super::super::support::{
    resource_async_lifecycle_rollback_workload, resource_branch_replay_workload,
};
use super::super::super::{
    resource_certification_builder, ObservationBoundaryOutcome, ObservedResourceNodeState,
    ResourceLifecycleClass, ResourceRequestId,
};

#[test]
fn resource_async_lifecycle_and_rollback_workload_preserves_committed_truth_and_suppresses_observation(
) {
    let outcome = resource_async_lifecycle_rollback_workload();

    assert_eq!(
        outcome.pre_rollback_replay.descriptor_digest(),
        outcome.post_rollback_replay.descriptor_digest(),
        "rollback lane must preserve descriptor truth exactly"
    );
    assert_eq!(
        outcome.pre_rollback_replay.lifecycle_digest(),
        outcome.post_rollback_replay.lifecycle_digest(),
        "rollback lane must preserve lifecycle truth exactly"
    );
    assert_eq!(
        outcome.pre_rollback_replay.output_continuity_digest(),
        outcome.post_rollback_replay.output_continuity_digest(),
        "rollback lane must preserve output continuity truth exactly"
    );
    assert_eq!(
        outcome.pre_rollback_replay.in_flight_digest(),
        outcome.post_rollback_replay.in_flight_digest(),
        "rollback lane must restore the same in-flight story"
    );
    assert_eq!(
        outcome.pre_rollback_replay.retry_lineage_digest(),
        outcome.post_rollback_replay.retry_lineage_digest(),
        "rollback lane must not leak retry-lineage drift"
    );
    assert_eq!(
        outcome.pre_rollback_replay.replay_digest(),
        outcome.post_rollback_replay.replay_digest(),
        "rollback lane must be indistinguishable from the control path where the failed completion never committed"
    );
    assert!(
        outcome.delivered_observations_after_rollback.is_empty(),
        "rollback-suppressed completion must not deliver observer packets"
    );
    assert_eq!(outcome.rollback_observation.events().len(), 1);
    assert_eq!(
        outcome.rollback_observation.events()[0].outcome(),
        ObservationBoundaryOutcome::RollbackSuppressed
    );
    assert_eq!(outcome.control_commit_observation.events().len(), 1);
    assert_eq!(
        outcome.control_commit_observation.events()[0].outcome(),
        ObservationBoundaryOutcome::Delivered
    );
    assert_eq!(
        outcome.rollback_observation.events()[0].observer_id(),
        outcome.control_commit_observation.events()[0].observer_id(),
        "rollback suppression must preserve observer identity exactly"
    );
    assert_eq!(
        outcome.rollback_observation.events()[0].handle_id(),
        outcome.control_commit_observation.events()[0].handle_id(),
        "rollback suppression must preserve observation handle identity exactly"
    );
    assert_eq!(
        outcome.rollback_observation.events()[0].policy(),
        outcome.control_commit_observation.events()[0].policy(),
        "rollback suppression must preserve observation policy exactly"
    );
    assert_eq!(
        outcome.rollback_observation.events()[0].visited(),
        outcome.control_commit_observation.events()[0].visited(),
        "rollback suppression must preserve visited classification exactly"
    );
    assert_eq!(
        outcome.rollback_observation.events()[0].recomputed(),
        outcome.control_commit_observation.events()[0].recomputed(),
        "rollback suppression must preserve recomputed classification exactly"
    );
    assert_eq!(
        outcome.rollback_observation.events()[0].meaningful_change(),
        outcome.control_commit_observation.events()[0].meaningful_change(),
        "rollback suppression must preserve meaningful-change classification exactly"
    );
    assert_eq!(
        outcome.rollback_observation.events()[0].trigger_matched(),
        outcome.control_commit_observation.events()[0].trigger_matched(),
        "rollback suppression must preserve trigger-match classification exactly"
    );
    assert_eq!(
        outcome.rollback_observation.events()[0]
            .matched_resource_nodes()
            .iter()
            .map(|node: &ObservedResourceNodeState| node.node())
            .collect::<Vec<_>>(),
        outcome.control_commit_observation.events()[0]
            .matched_resource_nodes()
            .iter()
            .map(|node: &ObservedResourceNodeState| node.node())
            .collect::<Vec<_>>(),
        "rollback suppression must preserve the same matched resource scope the no-failure control path would deliver"
    );
    assert_eq!(
        outcome.rollback_observation.events()[0].matched_resource_nodes()[0].lifecycle(),
        ResourceLifecycleClass::Pending
    );
    assert_eq!(
        outcome
            .delivered_observations_after_control_commit
            .len(),
        1,
        "the same completion should still deliver one observer packet on the no-failure control path"
    );
    assert_eq!(
        outcome.delivered_observations_after_control_commit[0].observer_id,
        outcome.control_commit_observation.events()[0]
            .observer_id()
            .get()
    );
    assert_eq!(
        outcome.delivered_observations_after_control_commit[0].handle_id,
        outcome.control_commit_observation.events()[0]
            .handle_id()
            .get()
    );
    assert_eq!(
        outcome.delivered_observations_after_control_commit[0].matched_node_count,
        outcome.control_commit_observation.events()[0]
            .matched_resource_nodes()
            .len()
    );
    assert_eq!(
        outcome.delivered_observations_after_control_commit[0].visited,
        outcome.control_commit_observation.events()[0].visited()
    );
    assert_eq!(
        outcome.delivered_observations_after_control_commit[0].recomputed,
        outcome.control_commit_observation.events()[0].recomputed()
    );
    assert_eq!(
        outcome.delivered_observations_after_control_commit[0].meaningful_change,
        outcome.control_commit_observation.events()[0].meaningful_change()
    );
    assert_eq!(
        outcome.delivered_observations_after_control_commit[0].trigger_matched,
        outcome.control_commit_observation.events()[0].trigger_matched()
    );
    assert_ne!(
        outcome.post_rollback_replay.lifecycle_digest(),
        outcome.control_path_replay.lifecycle_digest(),
        "control-path commit should move lifecycle truth beyond the rollback-preserved state"
    );
    assert_ne!(
        outcome.post_rollback_replay.replay_digest(),
        outcome.control_path_replay.replay_digest(),
        "control-path commit should append committed replay truth beyond the rollback-preserved lane"
    );
    assert!(!outcome
        .diagnostics_after_rollback
        .provenance_digest()
        .is_empty());
}

#[test]
fn resource_lifecycle_certification_rejects_non_equivalent_replay_truth() {
    let outcome = resource_branch_replay_workload(ResourceRequestId::new(9_991));

    let err = resource_certification_builder()
        .with_async_resource_lifecycle_parity(
            &outcome.feature.replay_after_restore,
            &outcome.feature.replay_after_snapshot_drift,
            &outcome.feature.diagnostics_after_restore,
            &outcome.feature.diagnostics_after_restore,
        )
        .expect_err("non-equivalent replay truth must not certify lifecycle parity");

    assert!(err
        .to_string()
        .contains("equivalent replay and diagnostics truth"));
}

#[test]
fn resource_rollback_certification_rejects_control_observation_mismatch() {
    let outcome = resource_async_lifecycle_rollback_workload();

    let err = resource_certification_builder()
        .with_async_rollback_observation_equivalence(
            outcome.rollback_report,
            outcome.rollback_observation.clone(),
            outcome.rollback_observation,
            &outcome.pre_rollback_replay,
            &outcome.post_rollback_replay,
            &outcome.diagnostics_after_rollback,
        )
        .expect_err(
            "rollback certification must reject a control path that is not a delivered packet",
        );

    assert!(err
        .to_string()
        .contains("requires only delivered events on the no-failure control path"));
}
