use crate::facade::{
    mark_dirty, ArtifactTransitionKind, ChangedRegion, DiagnosticsTier, EvaluationContext,
    LineageRecordKind, NodeContract, NodeEvaluationResult, OutputChange, PartitionSubscription,
    Recipe, ReplayEventKind, ReuseOrigin, SignalGraph, SignalRuntime, SignalRuntimePolicy,
    VersionComparatorPolicy,
};
use crate::tests::support::{version_ab, ASPECT_A, ASPECT_B};

#[test]
fn snapshot_restore_preserves_advanced_reuse_history_truth() {
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
    runtime.set_runtime_policy(SignalRuntimePolicy::development());
    let projection = runtime
        .define(Recipe {
            family: "phase5-advanced-reuse".into(),
            contract: NodeContract::reads([ASPECT_A])
                .with_produces([ASPECT_B])
                .with_cross_identity_persistent_matching()
                .with_partial_artifact_splicing()
                .with_partition_scope(PartitionSubscription::whole_partition("wing")),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("phase5-advanced-artifact")
                        .with_output_change(OutputChange::Refreshed)
                        .with_changed_region(ChangedRegion::new("wing")),
                ))
            },
        })
        .unwrap();
    let source = projection.keyed("source");
    let alias = projection.keyed("alias");
    let wing = projection.keyed("wing");
    let alias_node = alias.node(&mut runtime);
    let wing_node = wing.node(&mut runtime);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            source.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity_with_lineage_mapping(
                tx,
                "source",
                "shape-v1",
                "lineage-map:mesh-42->mesh-77",
            )
        })
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            wing.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    mark_dirty(runtime.graph_mut(), wing_node, ASPECT_A).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            wing.evaluate_partial_splice(
                tx,
                "shape-v1",
                [PartitionSubscription::whole_partition("wing")],
            )
        })
        .unwrap();

    let snapshot = runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");

    mark_dirty(runtime.graph_mut(), alias_node, ASPECT_A).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_memoized(tx, "shape-v2")
        })
        .unwrap();
    mark_dirty(runtime.graph_mut(), wing_node, ASPECT_A).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            wing.evaluate_memoized(tx, "shape-v2")
        })
        .unwrap();

    runtime.restore_snapshot(&snapshot).unwrap();

    let history = runtime
        .observe()
        .execution_history_summary(DiagnosticsTier::Development);
    let alias_summary = history
        .nodes
        .iter()
        .find(|node| node.node == alias_node)
        .expect("alias history summary");
    assert_eq!(
        alias_summary.reuse_origin,
        Some(ReuseOrigin::CrossIdentityPersistentReuse)
    );
    assert_eq!(
        alias_summary.persistent_correspondence_kind,
        Some(crate::data::reuse::PersistentCorrespondenceKind::LineageBackedMapping)
    );

    let wing_summary = history
        .nodes
        .iter()
        .find(|node| node.node == wing_node)
        .expect("wing history summary");
    assert_eq!(
        wing_summary.reuse_origin,
        Some(ReuseOrigin::PartialArtifactSplice)
    );
    assert_eq!(wing_summary.composition_region_count, 1);

    let alias_explain = runtime.observe().explain(alias_node).unwrap();
    assert_eq!(
        alias_explain.reuse_origin,
        Some(ReuseOrigin::CrossIdentityPersistentReuse)
    );
    let wing_explain = runtime.observe().explain(wing_node).unwrap();
    assert_eq!(
        wing_explain.reuse_origin,
        Some(ReuseOrigin::PartialArtifactSplice)
    );

    assert!(runtime.graph().replay_events().iter().any(|event| {
        event.kind == ReplayEventKind::SnapshotRestored
            && event.snapshot_id == Some(snapshot.meta.snapshot_id)
    }));
    let branch_replay = runtime
        .observe()
        .replay_for_branch(runtime.observe().current_branch().id);
    assert!(branch_replay.frames.iter().any(|event| {
        event.kind == ReplayEventKind::SnapshotRestored
            && event.snapshot_id == Some(snapshot.meta.snapshot_id)
    }));

    let alias_lineage = runtime.observe().lineage_chain_for_node(alias_node);
    assert!(alias_lineage.iter().any(|record| matches!(
        &record.kind,
        LineageRecordKind::ArtifactTransition {
            transition: ArtifactTransitionKind::CrossIdentityPersistentReuse {
                correspondence_kind:
                    crate::data::reuse::PersistentCorrespondenceKind::LineageBackedMapping
            },
            ..
        }
    )));
    let wing_lineage = runtime.observe().lineage_chain_for_node(wing_node);
    assert!(wing_lineage.iter().any(|record| matches!(
        &record.kind,
        LineageRecordKind::ArtifactTransition {
            transition: ArtifactTransitionKind::PartialArtifactSplice {
                composition_region_count: 1,
                recomputed_region_count: 1
            },
            ..
        }
    )));
}
