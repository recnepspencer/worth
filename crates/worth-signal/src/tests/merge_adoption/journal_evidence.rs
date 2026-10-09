use crate::data::graph::BranchStructuralDelta;
use crate::facade::{
    BranchMutationJournalSlice, BranchMutationLedger, NodeEvaluationResult, SignalGraph,
    SignalRuntime,
};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn branch_mutation_journal_captures_structural_dependency_snapshot_and_artifact_deltas() {
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
                        NodeEvaluationResult::from_version(version_ab(70, 0))
                            .with_output_identity("journal-upstream"),
                    )
                } else {
                    let version = view.read_aspect_version(upstream, ASPECT_A)?;
                    view.finish(
                        NodeEvaluationResult::from_version(version)
                            .with_output_identity("journal-downstream"),
                    )
                };
                Ok(result)
            })?;
            Ok(())
        })
        .unwrap();

    let record = runtime
        .graph()
        .branch_mutation_records()
        .into_iter()
        .find(|(node, _)| *node == downstream)
        .map(|(_, record)| record)
        .expect("downstream node should have structural mutation journal");

    assert!(record.structural_deltas.iter().any(|delta| matches!(
        delta,
        BranchStructuralDelta::DependencyTopologyChanged(topology)
            if topology.added_edges.len() == 1 && topology.removed_edges.is_empty()
    )));
    assert!(record.structural_deltas.iter().any(|delta| matches!(
        delta,
        BranchStructuralDelta::DependencySnapshotChanged(snapshot)
            if snapshot.next_entry_count >= 1 && snapshot.changed_entry_count >= 1
    )));
    assert!(record.structural_deltas.iter().any(|delta| matches!(
        delta,
        BranchStructuralDelta::RuntimeArtifactChanged(artifact)
            if artifact.next_output_hash.is_some() && artifact.next_reuse_basis.is_some()
    )));
}

#[test]
fn branch_mutation_journal_slice_preserves_structural_records_for_overlap_filtering() {
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
    let other = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(source, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(80, 0))
                        .with_output_identity("journal-source"),
                ))
            })?;
            tx.read(other, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(81, 0))
                        .with_output_identity("journal-other"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let mut ledger = BranchMutationLedger::default();
    ledger.absorb_records(runtime.graph().branch_mutation_records());
    let slice = ledger.structural_merge_journal();
    let filtered = BranchMutationJournalSlice {
        records: slice
            .records
            .iter()
            .filter(|record| record.node == source)
            .cloned()
            .collect(),
    };

    assert_eq!(slice.candidate_nodes().len(), 2);
    assert!(filtered.contains_node(source));
    assert!(!filtered.contains_node(other));
    assert!(filtered.records[0]
        .structural_deltas
        .iter()
        .any(|delta| matches!(delta, BranchStructuralDelta::RuntimeArtifactChanged(_))));
}

#[test]
fn merge_branch_uses_branch_local_mutation_scope_instead_of_whole_live_scan() {
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
    let shared_a = runtime.graph_mut().node().output_identity().build();
    let shared_b = runtime.graph_mut().node().output_identity().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared_a, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(11, 0))
                        .with_output_identity("shared-a"),
                ))
            })?;
            tx.read(shared_b, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(12, 0))
                        .with_output_identity("shared-b"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-narrow-scope").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();

    let source_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .graph_mut()
        .append_dependency(source_only, shared_a, ASPECT_A)
        .unwrap();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(source_only, &|view| {
                let upstream = view.read_aspect_version(shared_a, ASPECT_A)?;
                Ok(view.finish(
                    NodeEvaluationResult::from_version(upstream)
                        .with_output_identity("feature-narrow"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let result = runtime.merge_branch_raw(feature, main).unwrap();

    assert!(
        result.counters.final_candidate_breadth == result.planned_candidates.nodes.len() as u64,
        "tracked branch-local mutations should lower an explicit planned candidate set"
    );
    assert!(
        result.counters.source_slice_breadth >= result.counters.final_candidate_breadth,
        "candidate-node merge should stay bounded by the source journal slice"
    );
    assert!(
        result.records.len() < runtime.graph().active_node_count(),
        "narrow candidate scope should plan fewer nodes than the full live authority surface"
    );
}
