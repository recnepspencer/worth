use crate::facade::{
    mark_dirty, ArtifactTransitionKind, ChangedRegion, DiagnosticsTier, EvaluationContext,
    LineageRecordKind, NodeContract, NodeEvaluationResult, OutputChange, PartitionSubscription,
    Recipe, ReplayEventKind, ReuseCrossing, ReuseSource, SignalGraph, SignalRuntime,
    SignalRuntimePolicy, VersionComparatorPolicy,
};
use crate::tests::support::{version_ab, ASPECT_A, ASPECT_B};
use std::sync::atomic::{AtomicU32, Ordering};

#[test]
fn defined_computation_evaluate_partial_splice_uses_public_api() {
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
    let compute_calls = AtomicU32::new(0);
    let projection = runtime
        .define(Recipe {
            family: "projection".into(),
            contract: NodeContract::reads([ASPECT_A])
                .with_produces([ASPECT_B])
                .with_partial_artifact_splicing()
                .with_partition_scope(PartitionSubscription::whole_partition("wing")),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("splice-artifact")
                        .with_output_change(OutputChange::Refreshed)
                        .with_changed_region(ChangedRegion::new("wing")),
                ))
            },
        })
        .unwrap();
    let wing = projection.keyed("wing");
    let node = wing.node(&mut runtime);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            wing.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();

    mark_dirty(runtime.graph_mut(), node, ASPECT_A).unwrap();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            wing.evaluate_partial_splice(
                tx,
                "shape-v1",
                [PartitionSubscription::whole_partition("wing")],
            )
        })
        .unwrap();

    assert_eq!(compute_calls.load(Ordering::Relaxed), 1);
    let explanation = runtime.observe().explain(node).unwrap();
    let reuse_basis = explanation.reuse_basis.expect("partial splice reuse basis");
    assert_eq!(
        reuse_basis.strategy,
        Some(crate::data::reuse::ReuseStrategy::PartialArtifactSplicing)
    );
    assert_eq!(reuse_basis.source, ReuseSource::PartialComposition);
    assert_eq!(reuse_basis.crossing, ReuseCrossing::CompositionBoundary);
    assert_eq!(reuse_basis.partition_region_basis_count, 1);
    assert_eq!(
        explanation.reuse_origin,
        Some(crate::data::reuse::ReuseOrigin::PartialArtifactSplice)
    );
    let replay_event = runtime
        .graph()
        .replay_events()
        .iter()
        .rev()
        .find(|event| event.kind == ReplayEventKind::TaskApplied && event.node == Some(node))
        .expect("partial splice replay event");
    assert_eq!(
        replay_event.reuse_origin,
        Some(crate::data::reuse::ReuseOrigin::PartialArtifactSplice)
    );
    assert_eq!(replay_event.composition_region_count, Some(1));
    let history = runtime
        .observe()
        .execution_history_summary(DiagnosticsTier::Development);
    assert_eq!(
        history
            .reuse_origin_counts
            .get(&crate::data::reuse::ReuseOrigin::PartialArtifactSplice)
            .copied(),
        Some(1)
    );
    let history_entry = history
        .nodes
        .iter()
        .find(|entry| entry.node == node)
        .expect("partial splice history entry");
    assert_eq!(
        history_entry.reuse_origin,
        Some(crate::data::reuse::ReuseOrigin::PartialArtifactSplice)
    );
    assert_eq!(history_entry.composition_region_count, 1);
    let lineage = runtime.observe().lineage_chain_for_node(node);
    assert!(lineage.iter().any(|record| matches!(
        &record.kind,
        LineageRecordKind::ArtifactTransition {
            transition: ArtifactTransitionKind::PartialArtifactSplice {
                composition_region_count: 1,
                recomputed_region_count: 1
            },
            ..
        }
    )));
    assert_eq!(
        runtime
            .observe()
            .metrics()
            .evaluation
            .partial_artifact_splice_count,
        1
    );
}

#[test]
fn branch_local_partial_splice_rejection_preserves_main_mixed_provenance() {
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
    runtime
        .graph_mut()
        .set_runtime_policy(SignalRuntimePolicy::development());
    let projection = runtime
        .define(Recipe {
            family: "projection".into(),
            contract: NodeContract::reads([ASPECT_A])
                .with_produces([ASPECT_B])
                .with_partial_artifact_splicing()
                .with_partition_scope(PartitionSubscription::whole_partition("wing")),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("branch-splice-artifact")
                        .with_output_change(OutputChange::Refreshed)
                        .with_changed_region(ChangedRegion::new("wing")),
                ))
            },
        })
        .unwrap();
    let wing = projection.keyed("wing-branch");
    let node = wing.node(&mut runtime);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            wing.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    mark_dirty(runtime.graph_mut(), node, ASPECT_A).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            wing.evaluate_partial_splice(
                tx,
                "shape-v1",
                [PartitionSubscription::whole_partition("wing")],
            )
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let main_lineage_before = runtime.observe().lineage_chain_for_node(node);
    let main_replay_before = runtime.observe().replay_for_branch(main.id);

    let feature = runtime.create_branch("feature-partial-splice").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();
    mark_dirty(runtime.graph_mut(), node, ASPECT_A).unwrap();

    let err = runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            wing.evaluate_partial_splice(
                tx,
                "shape-v1",
                [PartitionSubscription::whole_partition("tail")],
            )
        })
        .expect_err("feature branch should reject changed composition regions");
    assert!(err.to_string().contains("reuse certification failed"));
    assert_eq!(
        runtime
            .observe()
            .metrics()
            .evaluation
            .reuse_rejected_boundary_mismatch_count,
        1
    );
    assert_eq!(
        runtime
            .graph()
            .observe()
            .materialize()
            .materialize_trace_summary(node)
            .unwrap()
            .and_then(|summary| summary.reuse_boundary_context)
            .and_then(|ctx| ctx.composition_regions().cloned())
            .map(|regions| regions.as_slice().len()),
        Some(1),
        "failed branch-local splice admission must preserve the last committed composition basis"
    );

    let feature_replay_after = runtime.observe().replay_for_branch(feature.id);
    assert!(
        feature_replay_after
            .frames
            .iter()
            .all(|frame| frame.branch_id == feature.id),
        "feature replay must remain branch-local after stale splice rejection"
    );

    runtime.switch_branch(main.clone()).unwrap();
    assert_eq!(
        runtime.observe().lineage_chain_for_node(node),
        main_lineage_before,
        "feature-branch splice rejection must not contaminate main lineage"
    );
    let main_replay_after = runtime.observe().replay_for_branch(main.id);
    assert_eq!(
        main_replay_after
            .frames
            .iter()
            .filter(|frame| frame.kind == ReplayEventKind::TaskApplied)
            .count(),
        main_replay_before
            .frames
            .iter()
            .filter(|frame| frame.kind == ReplayEventKind::TaskApplied)
            .count(),
        "feature-branch splice rejection must not append task-apply replay on main"
    );
    assert_eq!(
        main_replay_after
            .frames
            .iter()
            .filter(|frame| frame.kind == ReplayEventKind::TransactionCommitted)
            .count(),
        main_replay_before
            .frames
            .iter()
            .filter(|frame| frame.kind == ReplayEventKind::TransactionCommitted)
            .count(),
        "feature-branch splice rejection must not append committed execution replay on main"
    );
}

#[test]
fn branch_local_partial_splice_history_retains_committed_region_accounting_after_rejection() {
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
    let projection = runtime
        .define(Recipe {
            family: "projection".into(),
            contract: NodeContract::reads([ASPECT_A])
                .with_produces([ASPECT_B])
                .with_partial_artifact_splicing()
                .with_partition_scope(PartitionSubscription::whole_partition("wing")),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("branch-splice-history-artifact")
                        .with_output_change(OutputChange::Refreshed)
                        .with_changed_region(ChangedRegion::new("wing")),
                ))
            },
        })
        .unwrap();
    let wing = projection.keyed("wing-branch-history");
    let node = wing.node(&mut runtime);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            wing.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    mark_dirty(runtime.graph_mut(), node, ASPECT_A).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            wing.evaluate_partial_splice(
                tx,
                "shape-v1",
                [PartitionSubscription::whole_partition("wing")],
            )
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-partial-splice-history")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    mark_dirty(runtime.graph_mut(), node, ASPECT_A).unwrap();
    let _ = runtime.transaction(request_execution, &mut runtime_ctx, |tx| {
        wing.evaluate_partial_splice(
            tx,
            "shape-v1",
            [PartitionSubscription::whole_partition("tail")],
        )
    });

    let feature_history = runtime
        .observe()
        .execution_history_summary(DiagnosticsTier::Development);
    let feature_entry = feature_history
        .nodes
        .iter()
        .find(|entry| entry.node == node)
        .expect("feature history entry");
    assert_eq!(
        feature_entry.reuse_origin,
        Some(crate::data::reuse::ReuseOrigin::PartialArtifactSplice),
        "rejected splice evolution must not erase the last committed splice origin"
    );
    assert_eq!(
        feature_entry.composition_region_count, 1,
        "history should keep the committed composition region count after rejected branch-local splice evolution"
    );

    runtime.switch_branch(main.clone()).unwrap();
    let main_history = runtime
        .observe()
        .execution_history_summary(DiagnosticsTier::Development);
    let main_entry = main_history
        .nodes
        .iter()
        .find(|entry| entry.node == node)
        .expect("main history entry");
    assert_eq!(main_entry.composition_region_count, 1);
    let lineage = runtime.observe().lineage_chain_for_node(node);
    assert!(lineage.iter().any(|record| matches!(
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
