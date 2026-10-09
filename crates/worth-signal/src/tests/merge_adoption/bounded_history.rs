use crate::data::error::SignalError;
use crate::facade::{
    BranchMergeFailureKind, LineageRecordKind, MergeBoundaryWitnessKind, NodeEvaluationResult,
    ReplayEventKind, SignalGraph, SignalRuntime,
};
use crate::tests::support::version_ab;

#[test]
fn merge_branch_without_established_journal_boundary_fails_explicitly() {
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
    let mut runtime_ctx = ();
    let main = runtime.observe().current_branch();
    let shared = runtime.graph_mut().node().output_identity().build();
    let feature = runtime
        .create_branch("feature-missing-merge-boundary")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(140, 0))
                        .with_output_identity("missing-boundary-feature"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime
        .clear_branch_merge_boundary_for_test(feature.id)
        .expect("feature branch state should be present");

    runtime.switch_branch(main.clone()).unwrap();
    let err = runtime
        .merge_branch_raw(feature.clone(), main.clone())
        .expect_err("merge must fail explicitly when no bounded journal boundary exists");

    assert!(matches!(
        err,
        SignalError::BranchMergeFailed {
            kind: BranchMergeFailureKind::UnsupportedMergeStrategy,
            evidence: None,
            ..
        }
    ));
    assert!(
        err.to_string().contains("mutation-journal boundary"),
        "failure message should explain that bounded journal proof is required"
    );
}

#[test]
fn repeated_merge_after_target_restore_stays_bounded_and_history_honest() {
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
    let feature = runtime
        .create_branch("feature-repeated-restore-merge")
        .unwrap();
    let mut runtime_ctx = ();

    runtime.switch_branch(feature.clone()).unwrap();
    let first = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(first, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(96, 0))
                        .with_output_identity("restore-cycle-first"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let first_merge = runtime
        .merge_branch_raw(feature.clone(), main.clone())
        .unwrap();
    assert!(
        first_merge
            .records
            .iter()
            .any(|record| record.source_node == first),
        "first merge should include the initial source-only node"
    );
    let merged_snapshot = runtime.capture_branch_snapshot(main.clone()).unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    let second = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(second, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(97, 0))
                        .with_output_identity("restore-cycle-second"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let unrelated = runtime.graph_mut().node().output_identity().build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(unrelated, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(98, 0))
                        .with_output_identity("restore-cycle-unrelated"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let branch_merge_replay_before_restore = runtime
        .graph()
        .replay_events()
        .iter()
        .filter(|event| event.kind == ReplayEventKind::BranchMerged)
        .count();
    let branch_merge_lineage_before_restore = runtime
        .graph()
        .observe()
        .lineage_records()
        .iter()
        .filter(|record| matches!(record.kind, LineageRecordKind::BranchMerge { .. }))
        .count();

    runtime
        .restore_branch_snapshot(main.clone(), &merged_snapshot)
        .unwrap();

    assert_eq!(
        runtime
            .graph()
            .replay_events()
            .iter()
            .filter(|event| event.kind == ReplayEventKind::BranchMerged)
            .count(),
        branch_merge_replay_before_restore,
        "target restore between merge cycles must not fabricate extra branch merge replay events"
    );
    assert_eq!(
        runtime
            .graph()
            .observe()
            .lineage_records()
            .iter()
            .filter(|record| matches!(record.kind, LineageRecordKind::BranchMerge { .. }))
            .count(),
        branch_merge_lineage_before_restore,
        "target restore between merge cycles must not fabricate extra branch merge lineage"
    );

    let second_merge = runtime.merge_branch_raw(feature, main).unwrap();
    assert!(
        second_merge.counters.final_candidate_breadth
            == second_merge.planned_candidates.nodes.len() as u64,
        "repeated merge after restore should remain bounded to the branch mutation candidate set"
    );
    assert!(
        matches!(
            second_merge.boundary_witness.kind,
            MergeBoundaryWitnessKind::MutationJournalBoundary
        ),
        "repeated merge after restore must remain anchored to the mutation-journal witness"
    );
    assert!(
        second_merge
            .records
            .iter()
            .all(|record| record.source_node != first),
        "already-merged source nodes must stay retired after target restore"
    );
    assert!(
        second_merge
            .records
            .iter()
            .any(|record| record.source_node == second),
        "new source-side work should remain merge-visible after target restore"
    );
}
