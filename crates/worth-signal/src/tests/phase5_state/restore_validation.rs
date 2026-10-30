use crate::facade::{
    mark_dirty, LineageRecordKind, NodeEvaluationResult, ReplayEventKind, SignalError, SignalGraph,
    SignalRuntime,
};
use crate::tests::support::{evaluate, version_ab, ASPECT_A};

#[test]
fn branch_snapshot_restore_rejects_incompatible_storage_profile_and_preserves_branch_head() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let branch = runtime.create_branch("analysis").unwrap();
    let snapshot = runtime.capture_branch_snapshot(branch.clone()).unwrap();
    assert_eq!(
        runtime
            .observe()
            .branch_handle(branch.id)
            .unwrap()
            .head_snapshot_id,
        Some(snapshot.meta.snapshot_id),
        "capturing a branch snapshot should advance that branch head metadata"
    );
    let mut incompatible = snapshot.clone();
    incompatible.meta.core_storage_profile = "definitely-not-this-profile".to_string();

    let err = runtime.restore_branch_snapshot(branch.clone(), &incompatible);
    assert!(err.is_err());

    let restored = runtime.capture_branch_snapshot(branch).unwrap();
    assert_eq!(
        restored.meta.core_storage_profile,
        snapshot.meta.core_storage_profile
    );
}

#[test]
fn restore_branch_snapshot_rejects_cross_branch_payloads_and_keeps_catalog_consistent() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let feature = runtime.create_branch("feature-cross").unwrap();
    let main = runtime.observe().current_branch();
    let main_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");

    let err = runtime.restore_branch_snapshot(feature.clone(), &main_snapshot);
    assert!(
        matches!(err, Err(SignalError::IncompatibleSnapshot { .. })),
        "cross-branch restore should be rejected"
    );

    let feature_snapshot = runtime.capture_branch_snapshot(feature.clone()).unwrap();
    assert_eq!(feature_snapshot.meta.branch_id, feature.id);
    assert_eq!(
        runtime
            .observe()
            .branch_handle(feature.id)
            .unwrap()
            .head_snapshot_id,
        Some(feature_snapshot.meta.snapshot_id),
        "non-active branch snapshot capture should update shared branch-head metadata"
    );
    assert_eq!(
        runtime
            .observe()
            .branch_handle(main.id)
            .unwrap()
            .head_snapshot_id,
        Some(main_snapshot.meta.snapshot_id),
        "capturing another branch snapshot should not erase the active branch head"
    );
}

#[test]
fn repeated_snapshot_restore_loops_do_not_leak_non_restore_lineage_or_branch_state() {
    let mut graph = SignalGraph::new();
    let source = graph.node().output_identity().build();
    evaluate(&mut graph, source, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0)).with_output_identity("stable"))
    })
    .unwrap();
    let snapshot = graph.capture_snapshot();
    let baseline_artifact = graph.observe().current_lineage_artifact(source).unwrap();

    for _ in 0..8 {
        mark_dirty(&mut graph, source, ASPECT_A).unwrap();
        graph.restore_snapshot(&snapshot).unwrap();
    }

    assert_eq!(
        graph.observe().current_lineage_artifact(source),
        Some(baseline_artifact)
    );
    let lineage = graph.observe().lineage_records();
    assert!(
        lineage
            .iter()
            .filter(|record| matches!(record.kind, LineageRecordKind::SnapshotRestore { .. }))
            .count()
            == 8,
        "restore loops should preserve restore history instead of silently erasing it"
    );
    assert!(
        lineage.iter().all(|record| matches!(
            record.kind,
            LineageRecordKind::SnapshotRestore { .. }
        ) || record.node() == Some(source)),
        "restore churn should not create stray lineage ownership"
    );
    assert!(
        graph
            .replay_events()
            .iter()
            .filter(|event| event.kind == ReplayEventKind::SnapshotRestored)
            .count()
            == 8,
        "restore churn should preserve restore replay history instead of silently erasing it"
    );
}
