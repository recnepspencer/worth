use crate::facade::{
    compare_lineage_records, compare_replay_slices, lineage_records_equivalent, mark_dirty,
    replay_slices_equivalent, ArtifactTransitionKind, LineageRecordKind, NodeEvaluationResult,
    SignalGraph, SignalRuntime, SignalRuntimePolicy, SignalSnapshotMeta,
    SnapshotRestoreLineageMode,
};
use crate::tests::support::{evaluate, version_ab, ASPECT_A};

#[test]
fn lineage_chain_preserves_invalidation_and_restore_events_for_the_same_artifact() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(
        SignalRuntimePolicy::forensic()
            .with_snapshot_restore_lineage_mode(SnapshotRestoreLineageMode::PerNode),
    );
    let source = graph.node().output_identity().build();

    evaluate(&mut graph, source, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0)).with_output_identity("stable"))
    })
    .unwrap();
    let artifact_id = graph
        .observe()
        .current_lineage_artifact(source)
        .expect("materialized node should have lineage");

    mark_dirty(&mut graph, source, ASPECT_A).unwrap();
    let snapshot = graph.capture_snapshot();
    graph.restore_snapshot(&snapshot).unwrap();

    let chain = graph.observe().lineage_chain_for_artifact(artifact_id);
    assert!(
        chain.iter().any(|record| matches!(
            record.kind,
            LineageRecordKind::ArtifactTransition {
                transition: ArtifactTransitionKind::Replaced,
                ..
            }
        )),
        "lineage chain should include the artifact's original materialization"
    );
    assert!(
        chain
            .iter()
            .any(|record| matches!(record.kind, LineageRecordKind::Invalidation { .. })),
        "lineage chain should retain invalidation history for the same artifact"
    );
    assert!(
        chain
            .iter()
            .any(|record| matches!(record.kind, LineageRecordKind::SnapshotRestore { .. })),
        "lineage chain should retain restore history for the same artifact"
    );
}

#[test]
fn snapshot_metadata_and_replay_ranges_are_inspectable_without_restore() {
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
    let node = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(node, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("range-artifact"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let before = runtime
        .replay_for_branch(runtime.observe().current_branch().id)
        .frames
        .last()
        .map(|frame| frame.cursor)
        .expect("replay should exist after first transaction");

    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    assert_eq!(snapshot.meta().snapshot_id, snapshot.snapshot_id());
    assert_eq!(snapshot.meta().branch_id, snapshot.branch_id());
    assert_eq!(
        snapshot.meta().schema_version,
        SignalSnapshotMeta::SCHEMA_VERSION
    );

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(node, ASPECT_A)?;
            tx.read(node, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(2, 0))
                        .with_output_identity("range-artifact-2"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let current_branch = runtime.observe().current_branch().id;
    let branch_replay = runtime.observe().replay_for_branch(current_branch);
    let end = branch_replay
        .frames
        .last()
        .map(|frame| frame.cursor)
        .expect("replay tail should exist");
    let ranged = runtime.observe().replay_between(before, end);
    assert!(
        !ranged.frames.is_empty(),
        "bounded replay range should return frames within the requested cursor window"
    );
    assert!(
        ranged
            .frames
            .iter()
            .all(|frame| frame.cursor >= before && frame.cursor <= end),
        "bounded replay ranges should respect both cursor endpoints"
    );
}

#[test]
fn replay_and_lineage_diff_helpers_compare_generic_phase5_artifacts() {
    let mut graph = SignalGraph::new();
    let source = graph.node().output_identity().build();

    evaluate(&mut graph, source, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0)).with_output_identity("diff-a"))
    })
    .unwrap();
    let left_replay = graph.observe().replay_for_node(source).to_owned_slice();
    let left_lineage = graph.observe().lineage_for_node(source).to_owned_records();

    let right_replay = graph.observe().replay_for_node(source).to_owned_slice();
    let right_lineage = graph.observe().lineage_for_node(source).to_owned_records();

    assert!(replay_slices_equivalent(&left_replay, &right_replay));
    assert!(compare_replay_slices(&left_replay, &right_replay).is_empty());
    assert!(lineage_records_equivalent(&left_lineage, &right_lineage));
    assert!(compare_lineage_records(&left_lineage, &right_lineage).is_empty());

    mark_dirty(&mut graph, source, ASPECT_A).unwrap();
    graph.capture_snapshot();
    let changed_replay = graph
        .observe()
        .replay_for_branch(graph.observe().current_branch().id)
        .to_owned_slice();
    let changed_lineage = graph.observe().lineage_for_node(source).to_owned_records();
    assert!(!compare_replay_slices(&left_replay, &changed_replay).is_empty());
    assert!(!compare_lineage_records(&left_lineage, &changed_lineage).is_empty());
}
