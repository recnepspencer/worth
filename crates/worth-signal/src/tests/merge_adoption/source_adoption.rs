use crate::facade::{
    ArtifactAuthorityClass, ArtifactMergeAction, ArtifactMergeAuthority, ConflictMergePolicy,
    LineageRecordKind, MergeAdoptability, MergeBoundaryWitnessKind, NodeEvaluationResult,
    ReplayEventKind, SignalGraph, SignalRuntime,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn merge_branch_introduces_source_only_node_with_new_target_id_and_merge_traces() {
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
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("shared-main"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-introduce").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();

    let source_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .graph_mut()
        .append_dependency(source_only, shared, ASPECT_A)
        .unwrap();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(source_only, &|view| {
                let upstream = view.read_aspect_version(shared, ASPECT_A)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(upstream)
                        .with_output_identity("feature-only"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let source_artifact_id = runtime
        .observe()
        .current_lineage_artifact(source_only)
        .expect("source-only node should materialize a lineage artifact");

    runtime.switch_branch(main.clone()).unwrap();
    let main_node_count_before = runtime.graph().active_node_count();

    let result = runtime
        .merge_branch_raw(feature.clone(), main.clone())
        .unwrap();
    assert_eq!(
        result.reconciliation_policy.conflict,
        ConflictMergePolicy::ResolveSourceStateWhenStructureMatches
    );
    let introduced = result
        .records
        .iter()
        .find(|record| {
            record.source_node == source_only
                && matches!(record.action, ArtifactMergeAction::IntroducedIntoTarget)
        })
        .expect("merge should report the introduced source-only node");
    let introduced_target = introduced
        .target_node
        .expect("introduced source-only node should allocate a target node");

    assert_ne!(
        introduced_target, source_only,
        "source-only adoption must allocate a distinct target node id"
    );
    assert!(
        runtime.graph().is_alive(introduced_target),
        "introduced target node should exist on the merged target branch"
    );
    assert_eq!(
        runtime.graph().active_node_count(),
        main_node_count_before + 1,
        "merging a source-only node into the active target should increase target node count"
    );
    assert_eq!(
        runtime
            .graph()
            .dependencies_of(introduced_target)
            .unwrap()
            .iter()
            .map(|edge| edge.source())
            .collect::<Vec<_>>(),
        vec![shared],
        "introduced target node dependencies should be remapped to target authority ids"
    );
    assert_ne!(
        introduced.target_artifact_id_after,
        Some(source_artifact_id),
        "introduced target nodes must not reuse the source branch artifact lineage id by default"
    );
    assert!(
        runtime
            .graph()
            .replay_events()
            .iter()
            .any(|event| event.kind == ReplayEventKind::BranchMerged),
        "merge should emit a branch-level replay boundary"
    );
    assert!(
        runtime
            .graph()
            .observe()
            .lineage_records()
            .iter()
            .any(|record| matches!(record.kind, LineageRecordKind::BranchMerge { .. })),
        "merge should emit branch merge lineage"
    );
    assert!(
        runtime
            .graph()
            .observe()
            .lineage_records()
            .iter()
            .any(|record| matches!(record.kind, LineageRecordKind::ArtifactMerge { .. })),
        "merge should emit artifact merge lineage"
    );
}

#[test]
fn merge_branch_introduces_multiple_source_only_nodes_with_internal_dependencies() {
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
    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-chain").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();

    let upstream = runtime.graph_mut().node().output_identity().build();
    let downstream = runtime.graph_mut().node().output_identity().build();
    runtime
        .graph_mut()
        .append_dependency(downstream, upstream, ASPECT_A)
        .unwrap();

    let mut runtime_ctx = ();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(downstream, &|view| {
                let result = if view.node() == upstream {
                    view.finish(
                        NodeEvaluationResult::from_version(version_ab(3, 0))
                            .with_output_identity("feature-upstream"),
                    )
                } else {
                    let version = view.read_aspect_version(upstream, ASPECT_A)?;
                    view.finish(
                        NodeEvaluationResult::from_version(version)
                            .with_output_identity("feature-downstream"),
                    )
                };
                Ok(result)
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let result = runtime.merge_branch_raw(feature, main).unwrap();

    let introduced_upstream = result
        .records
        .iter()
        .find(|record| record.source_node == upstream)
        .and_then(|record| record.target_node)
        .expect("upstream node should be introduced into target");
    let introduced_downstream = result
        .records
        .iter()
        .find(|record| record.source_node == downstream)
        .and_then(|record| record.target_node)
        .expect("downstream node should be introduced into target");

    assert_eq!(
        runtime
            .graph()
            .dependencies_of(introduced_downstream)
            .unwrap()
            .iter()
            .map(|edge| edge.source())
            .collect::<Vec<_>>(),
        vec![introduced_upstream],
        "introduced internal dependencies must remap to introduced target node ids"
    );
}

#[test]
fn merge_branch_skips_non_adoptable_source_only_nodes() {
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
    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-non-adoptable").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();

    let source_only = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(source_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(5, 0))
                        .with_output_identity("non-adoptable"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    {
        let mut graph = runtime.graph_mut();
        let mut entry = graph.get_entry_mut(source_only).unwrap();
        let mut runtime_artifact = entry
            .get_runtime_artifact_state()
            .cloned()
            .expect("source-only node should have runtime artifact state");
        runtime_artifact.warm_mut().merge_authority = ArtifactMergeAuthority {
            authority_class: ArtifactAuthorityClass::BranchLocalSpeculative,
            adoptability: MergeAdoptability::NonAdoptableBranchLocal,
        };
        entry.set_runtime_artifact_state(Some(runtime_artifact));
    }

    runtime.switch_branch(main.clone()).unwrap();
    let main_node_count_before = runtime.graph().active_node_count();
    let result = runtime.merge_branch_raw(feature, main).unwrap();
    let skipped = result
        .records
        .iter()
        .find(|record| record.source_node == source_only)
        .expect("merge should still report the skipped source-only node");

    assert!(
        matches!(skipped.action, ArtifactMergeAction::SkippedNonAdoptable),
        "non-adoptable source-only nodes should be skipped explicitly"
    );
    assert_eq!(
        skipped.target_node, None,
        "skipped source-only nodes must not allocate a target identity"
    );
    assert_eq!(
        runtime.graph().active_node_count(),
        main_node_count_before,
        "skipped non-adoptable nodes must not change target authority breadth"
    );
}

#[test]
fn merge_branch_counters_and_summary_surface_match_introduced_work() {
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
    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-counters").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();

    let source_only = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(source_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(7, 0))
                        .with_output_identity("counter-node"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main).unwrap();
    let result = runtime
        .merge_branch_raw(feature, runtime.observe().current_branch())
        .unwrap();

    assert_eq!(
        result.counters.introduced_node_count, 1,
        "merge counters should expose introduced-node adoption work"
    );
    assert_eq!(
        result.counters.replay_event_count, 1,
        "merge counters should reflect the branch-level replay boundary"
    );
    assert_eq!(
        result.counters.merge_lineage_record_count,
        (result.records.len() + 1) as u64,
        "merge counters should account for one branch merge record plus per-node artifact merge records"
    );
    assert!(
        matches!(
            result.boundary_witness.kind,
            MergeBoundaryWitnessKind::MutationJournalBoundary
        ),
        "tracked branch-local mutation scope should surface a bounded merge witness"
    );
}
