use crate::facade::{
    BranchMergeDivergence, BranchMergeKind, LineageRecordKind, NodeEvaluationResult,
    ReplayEventKind, SignalGraph, SignalRuntime,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn restore_branch_snapshot_after_merge_preserves_introduced_nodes_and_remapped_dependencies() {
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
    let shared = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(90, 0))
                        .with_output_identity("restore-shared"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-restore-merge").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();

    let upstream = runtime.graph_mut().node().output_identity().build();
    let downstream = runtime.graph_mut().node().output_identity().build();
    runtime
        .graph_mut()
        .append_dependency(upstream, shared, ASPECT_A)
        .unwrap();
    runtime
        .graph_mut()
        .append_dependency(downstream, upstream, ASPECT_A)
        .unwrap();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(downstream, &|view| {
                let result = if view.node() == upstream {
                    let version = view.read_aspect_version(shared, ASPECT_A)?;
                    view.finish(
                        NodeEvaluationResult::from_version(version)
                            .with_output_identity("restore-upstream"),
                    )
                } else {
                    let version = view.read_aspect_version(upstream, ASPECT_A)?;
                    view.finish(
                        NodeEvaluationResult::from_version(version)
                            .with_output_identity("restore-downstream"),
                    )
                };
                Ok(result)
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let merge = runtime.merge_branch_raw(feature, main.clone()).unwrap();
    let introduced_upstream = merge
        .records
        .iter()
        .find(|record| record.source_node == upstream)
        .and_then(|record| record.target_node)
        .expect("merged upstream node should be introduced into target");
    let introduced_downstream = merge
        .records
        .iter()
        .find(|record| record.source_node == downstream)
        .and_then(|record| record.target_node)
        .expect("merged downstream node should be introduced into target");

    let merged_snapshot = runtime.capture_branch_snapshot(main.clone()).unwrap();

    let unrelated = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(unrelated, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(91, 0))
                        .with_output_identity("post-merge-unrelated"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime
        .restore_branch_snapshot(main.clone(), &merged_snapshot)
        .unwrap();

    assert!(
        runtime.graph().is_alive(introduced_upstream),
        "restoring the merged branch snapshot should preserve introduced upstream nodes"
    );
    assert!(
        runtime.graph().is_alive(introduced_downstream),
        "restoring the merged branch snapshot should preserve introduced downstream nodes"
    );
    assert!(!runtime.graph().is_alive(unrelated));
    assert_eq!(
        runtime
            .graph()
            .dependencies_of(introduced_downstream)
            .unwrap()
            .iter()
            .map(|edge| edge.source())
            .collect::<Vec<_>>(),
        vec![introduced_upstream],
        "restored merged topology must retain remapped target dependencies"
    );
}

#[test]
fn restore_after_merge_does_not_emit_false_branch_merge_history() {
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
                    NodeEvaluationResult::from_version(version_ab(92, 0))
                        .with_output_identity("history-shared"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-history-restore").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(93, 0))
                        .with_output_identity("history-feature-only"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    runtime.merge_branch_raw(feature, main.clone()).unwrap();
    let merged_snapshot = runtime.capture_branch_snapshot(main.clone()).unwrap();

    let branch_merge_replay_before = runtime
        .graph()
        .replay_events()
        .iter()
        .filter(|event| event.kind == ReplayEventKind::BranchMerged)
        .count();
    let branch_merge_lineage_before = runtime
        .graph()
        .observe()
        .lineage_records()
        .iter()
        .filter(|record| matches!(record.kind, LineageRecordKind::BranchMerge { .. }))
        .count();

    runtime
        .restore_branch_snapshot(main, &merged_snapshot)
        .unwrap();

    let branch_merge_replay_after = runtime
        .graph()
        .replay_events()
        .iter()
        .filter(|event| event.kind == ReplayEventKind::BranchMerged)
        .count();
    let branch_merge_lineage_after = runtime
        .graph()
        .observe()
        .lineage_records()
        .iter()
        .filter(|record| matches!(record.kind, LineageRecordKind::BranchMerge { .. }))
        .count();

    assert_eq!(
        branch_merge_replay_after, branch_merge_replay_before,
        "snapshot restore after merge must not fabricate extra BranchMerged replay events"
    );
    assert_eq!(
        branch_merge_lineage_after, branch_merge_lineage_before,
        "snapshot restore after merge must not fabricate extra BranchMerge lineage records"
    );
    assert!(
        runtime.graph().replay_events().iter().any(|event| {
            event.kind == ReplayEventKind::SnapshotRestored
                && event.snapshot_id == Some(merged_snapshot.meta.snapshot_id)
        }),
        "restore should still emit its own snapshot restore replay boundary"
    );
}

#[test]
fn active_restore_reinstates_branch_merge_ledger_boundary_for_later_fast_forward_merge() {
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
    let shared = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(94, 0))
                        .with_output_identity("restore-base-shared"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let base_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-active-restore-fast-forward")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(94, 0))
                        .with_output_identity("restore-base-shared"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(95, 0))
                        .with_output_identity("restore-main-advanced"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.restore_snapshot(&base_snapshot).unwrap();

    let result = runtime.merge_branch_raw(feature, main).unwrap();
    assert_eq!(
        result.merge_kind,
        BranchMergeKind::FastForward,
        "restoring the active branch snapshot must reinstate the captured merge boundary and avoid stale target divergence"
    );
    assert_eq!(
        result.divergence,
        BranchMergeDivergence::None,
        "active restore should clear stale target overlap from the restored branch ledger"
    );
}
