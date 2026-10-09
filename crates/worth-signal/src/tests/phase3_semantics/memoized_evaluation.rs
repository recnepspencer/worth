use crate::facade::{
    mark_dirty, DiagnosticsTier, EvaluationContext, MemoizedResultOrigin, NodeContract,
    NodeEvaluationResult, OutputChange, Recipe, ReplayEventKind, ReuseCrossing, ReuseSource,
    SignalGraph, SignalRuntime, VersionComparatorPolicy,
};
use crate::tests::support::{define_keyed_computation, version_ab, ASPECT_A, ASPECT_B};
use std::sync::atomic::{AtomicU32, Ordering};

#[test]
fn keyed_evaluation_can_reuse_memoized_result() {
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
    let family = define_keyed_computation(&mut runtime, "projection", ());
    let keyed = family.keyed("bulkhead");
    let node = keyed.node(&mut runtime);
    let computation = keyed.memoized("shape-v1");
    let mut runtime_ctx = ();
    let compute_calls = AtomicU32::new(0);

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.evaluate_keyed(node, &computation, &|view| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("bulkhead-artifact")
                        .with_output_change(OutputChange::Refreshed),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    mark_dirty(runtime.graph_mut(), node, ASPECT_A).unwrap();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.evaluate_keyed(node, &computation, &|view| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(99, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    assert_eq!(compute_calls.load(Ordering::Relaxed), 1);
    let explanation = runtime.observe().explain(node).unwrap();
    let reuse_basis = explanation.reuse_basis.expect("memoized reuse basis");
    assert_eq!(
        reuse_basis.strategy,
        Some(crate::data::reuse::ReuseStrategy::MemoizedArtifactReuse)
    );
    assert_eq!(reuse_basis.source, ReuseSource::MemoizedArtifact);
    assert_eq!(reuse_basis.crossing, ReuseCrossing::None);
    assert!(reuse_basis.dependency_snapshot_basis.is_some());
    assert_eq!(
        explanation.memoized_origin,
        Some(MemoizedResultOrigin::MemoizedFromCache)
    );
    let metrics = runtime.observe().metrics();
    assert_eq!(metrics.invalidation.keyed_evaluation_count, 2);
    assert_eq!(metrics.evaluation.memoization_hits, 1);
    assert_eq!(metrics.evaluation.memoization_misses, 1);
}

#[test]
fn defined_computation_evaluate_memoized_reuses_cached_result() {
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
            contract: NodeContract::reads([ASPECT_A]).with_produces([ASPECT_B]),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("bulkhead-artifact")
                        .with_output_change(OutputChange::Refreshed),
                ))
            },
        })
        .unwrap();
    let bulkhead = projection.keyed("bulkhead");
    let node = bulkhead.node(&mut runtime);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            bulkhead.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();

    mark_dirty(runtime.graph_mut(), node, ASPECT_A).unwrap();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            bulkhead.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();

    assert_eq!(compute_calls.load(Ordering::Relaxed), 1);
    let explanation = runtime.observe().explain(node).unwrap();
    let reuse_basis = explanation.reuse_basis.expect("memoized reuse basis");
    assert_eq!(
        reuse_basis.strategy,
        Some(crate::data::reuse::ReuseStrategy::MemoizedArtifactReuse)
    );
    assert_eq!(reuse_basis.source, ReuseSource::MemoizedArtifact);
    assert_eq!(reuse_basis.crossing, ReuseCrossing::None);
    assert!(reuse_basis.dependency_snapshot_basis.is_some());
    assert_eq!(
        explanation.memoized_origin,
        Some(MemoizedResultOrigin::MemoizedFromCache)
    );
}

#[test]
fn defined_computation_evaluate_cross_identity_reuses_cached_result_via_public_api() {
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
                .with_cross_identity_persistent_matching(),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("cross-identity-artifact")
                        .with_output_change(OutputChange::Refreshed),
                ))
            },
        })
        .unwrap();
    let source = projection.keyed("source");
    let alias = projection.keyed("alias");
    let alias_node = alias.node(&mut runtime);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            source.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity(tx, "source", "shape-v1", "mesh-001")
        })
        .unwrap();

    assert_eq!(compute_calls.load(Ordering::Relaxed), 1);
    let explanation = runtime.observe().explain(alias_node).unwrap();
    let reuse_basis = explanation.reuse_basis.expect("cross-identity reuse basis");
    assert_eq!(
        reuse_basis.strategy,
        Some(crate::data::reuse::ReuseStrategy::CrossIdentityPersistentMatch)
    );
    assert_eq!(reuse_basis.source, ReuseSource::PersistentCorrespondence);
    assert_eq!(
        reuse_basis.crossing,
        ReuseCrossing::PersistentIdentityBoundary
    );
    assert_eq!(
        explanation.reuse_origin,
        Some(crate::data::reuse::ReuseOrigin::CrossIdentityPersistentReuse)
    );
    let replay_event = runtime
        .graph()
        .replay_events()
        .iter()
        .rev()
        .find(|event| event.kind == ReplayEventKind::TaskApplied && event.node == Some(alias_node))
        .expect("cross-identity replay event");
    assert_eq!(
        replay_event.reuse_origin,
        Some(crate::data::reuse::ReuseOrigin::CrossIdentityPersistentReuse)
    );
    let history = runtime
        .observe()
        .execution_history_summary(DiagnosticsTier::Development);
    assert_eq!(
        history
            .reuse_origin_counts
            .get(&crate::data::reuse::ReuseOrigin::CrossIdentityPersistentReuse)
            .copied(),
        Some(1)
    );
    assert!(history.nodes.iter().any(|node| {
        node.node == alias_node
            && node.reuse_origin
                == Some(crate::data::reuse::ReuseOrigin::CrossIdentityPersistentReuse)
    }));
    assert_eq!(
        runtime
            .observe()
            .metrics()
            .evaluation
            .cross_identity_reuse_count,
        1
    );
}
