use crate::facade::{
    LineageRecordKind, NodeEvaluationResult, SignalGraph, SignalRuntime, SignalRuntimePolicy,
    SnapshotRestoreLineageMode,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn snapshot_restore_lineage_defaults_to_compact_global_but_forensic_can_emit_per_node() {
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
    let dependent = runtime.graph_mut().node().build();
    runtime
        .graph_mut()
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(dependent, &|view| {
                let result = if view.node() == source {
                    view.finish(
                        NodeEvaluationResult::from_version(version_ab(1, 0))
                            .with_output_identity("compact-restore"),
                    )
                } else {
                    let version = view.read_aspect_version(source, ASPECT_A)?;
                    view.finish(NodeEvaluationResult::from_version(version))
                };
                Ok(result)
            })?;
            Ok(())
        })
        .unwrap();

    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    runtime.restore_snapshot(&snapshot).unwrap();
    let compact_restores = runtime
        .graph()
        .observe()
        .lineage_records()
        .iter()
        .filter(|record| {
            matches!(
                record.kind,
                LineageRecordKind::SnapshotRestore { snapshot_id, .. }
                    if snapshot_id == snapshot.meta.snapshot_id
            )
        })
        .count();
    assert_eq!(
        compact_restores, 1,
        "operational/development restore lineage should default to one compact global restore event"
    );

    runtime.set_runtime_policy(
        SignalRuntimePolicy::forensic()
            .with_snapshot_restore_lineage_mode(SnapshotRestoreLineageMode::PerNode),
    );
    let forensic_snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    runtime.restore_snapshot(&forensic_snapshot).unwrap();
    let forensic_restores = runtime
        .graph()
        .observe()
        .lineage_records()
        .iter()
        .filter(|record| {
            matches!(
                record.kind,
                LineageRecordKind::SnapshotRestore { snapshot_id, .. }
                    if snapshot_id == forensic_snapshot.meta.snapshot_id
            )
        })
        .count();
    assert!(
        forensic_restores >= 2,
        "forensic per-node restore lineage should emit restored entries for materialized artifacts"
    );
}
