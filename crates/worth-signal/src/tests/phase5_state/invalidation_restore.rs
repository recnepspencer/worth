use crate::facade::{
    mark_dirty, InvalidationCause, LineageRecordKind, NodeEvaluationResult, ReplayEventKind,
    SignalGraph, SignalRuntime, SignalRuntimePolicy, SnapshotRestoreKind,
    SnapshotRestoreLineageMode,
};
use crate::tests::support::{evaluate, version_ab, ASPECT_A};

#[test]
fn invalidation_emits_lineage_without_replacement_and_branch_restore_is_local() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let source = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("artifact-main"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main_branch = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-b").unwrap();
    let main_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    runtime.switch_branch(feature.clone()).unwrap();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(source, ASPECT_A)?;
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("artifact-feature"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let feature_snapshot = runtime.capture_branch_snapshot(feature.clone()).unwrap();
    mark_dirty(runtime.graph_mut(), source, ASPECT_A).unwrap();
    assert!(
        runtime
            .graph()
            .observe()
            .lineage_for_node(source)
            .iter()
            .any(|record| {
                matches!(
                    &record.kind,
                    LineageRecordKind::Invalidation {
                        cause: InvalidationCause::SourceAspectChanged { .. }
                            | InvalidationCause::DirectDependencyChanged { .. }
                            | InvalidationCause::TransitiveDependencyChanged { .. }
                            | InvalidationCause::PendingDependencyRevalidation { .. },
                        ..
                    }
                )
            }),
        "invalidation should record lineage even before the artifact is replaced"
    );

    runtime
        .restore_branch_snapshot(feature.clone(), &feature_snapshot)
        .unwrap();
    let feature_replay = runtime.observe().replay_for_branch(feature.id);
    assert!(
        feature_replay
            .frames
            .iter()
            .any(|event| event.kind == ReplayEventKind::SnapshotRestored),
        "branch-local restore should be visible in the restored branch replay stream"
    );

    runtime.switch_branch(main_branch).unwrap();
    assert_eq!(
        runtime
            .graph()
            .get_entry(source)
            .unwrap()
            .get_aspect_version()
            .get(ASPECT_A),
        1,
        "branch restore and branch-local churn must not contaminate main"
    );
    let main_replay = runtime
        .observe()
        .replay_around_snapshot(main_snapshot.meta.snapshot_id);
    assert!(
        main_replay
            .frames
            .iter()
            .all(|event| event.branch_id == runtime.observe().current_branch().id),
        "snapshot inspection on main should stay branch-local"
    );
}

#[test]
fn snapshot_restore_lineage_records_restore_mode_structurally() {
    let mut graph = SignalGraph::new();
    let source = graph.node().output_identity().build();

    evaluate(&mut graph, source, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0)).with_output_identity("stable"))
    })
    .unwrap();
    let snapshot = graph.capture_snapshot();
    graph.restore_snapshot(&snapshot).unwrap();

    let compact_restore = graph
        .observe()
        .lineage_records()
        .iter()
        .find(|record| record.snapshot_id() == Some(snapshot.meta.snapshot_id))
        .expect("compact restore lineage should be recorded");
    assert!(matches!(
        compact_restore.kind,
        LineageRecordKind::SnapshotRestore {
            restore_kind: SnapshotRestoreKind::CompactGlobal,
            ..
        }
    ));

    let mut forensic = SignalGraph::new();
    forensic.set_runtime_policy(
        SignalRuntimePolicy::development()
            .with_snapshot_restore_lineage_mode(SnapshotRestoreLineageMode::PerNode),
    );
    let node = forensic.node().output_identity().build();
    evaluate(&mut forensic, node, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0)).with_output_identity("stable"))
    })
    .unwrap();
    let per_node_snapshot = forensic.capture_snapshot();
    forensic.restore_snapshot(&per_node_snapshot).unwrap();

    let per_node_restore = forensic
        .observe()
        .lineage_records()
        .iter()
        .find(|record| {
            record.snapshot_id() == Some(per_node_snapshot.meta.snapshot_id)
                && record.node() == Some(node)
        })
        .expect("per-node restore lineage should be recorded");
    assert!(matches!(
        per_node_restore.kind,
        LineageRecordKind::SnapshotRestore {
            restore_kind: SnapshotRestoreKind::PerNodeArtifact,
            ..
        }
    ));
}

#[test]
fn snapshot_restore_lineage_uses_installed_authority_not_request_mirror() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(
        SignalRuntimePolicy::development()
            .with_snapshot_restore_lineage_mode(SnapshotRestoreLineageMode::PerNode),
    );
    let node = graph.node().output_identity().build();
    evaluate(&mut graph, node, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0)).with_output_identity("stable"))
    })
    .unwrap();
    let mut snapshot = graph.capture_snapshot();

    snapshot
        .checkpoint_image
        .authority
        .diagnostics
        .set_request_mirror(
            SignalRuntimePolicy::operational()
                .with_snapshot_restore_lineage_mode(SnapshotRestoreLineageMode::CompactGlobal),
        );
    graph
        .diagnostics_state_mut()
        .set_request_mirror(SignalRuntimePolicy::operational());
    graph
        .try_set_runtime_policy(
            SignalRuntimePolicy::operational()
                .with_snapshot_restore_lineage_mode(SnapshotRestoreLineageMode::CompactGlobal),
        )
        .expect("the conflicting live policy should be admissible");
    graph.restore_snapshot(&snapshot).unwrap();

    assert!(graph.observe().lineage_records().iter().any(|record| {
        record.snapshot_id() == Some(snapshot.meta.snapshot_id)
            && record.node() == Some(node)
            && matches!(
                record.kind,
                LineageRecordKind::SnapshotRestore {
                    restore_kind: SnapshotRestoreKind::PerNodeArtifact,
                    ..
                }
            )
    }));
}
