use crate::facade::*;
use crate::tests::support::*;

mod leased_execution;

#[test]
fn direct_whole_partition_commit_admits_matching_partition_causes() {
    let mut graph = SignalGraph::new();
    let source = graph.node().partitioned_output().build();
    evaluate(&mut graph, source, &mut |_id, _graph| Ok(version_ab(1, 0))).unwrap();
    let whole = graph.node().build();
    let detail = graph.node().build();
    graph
        .append_partition_dependency(whole, source, ASPECT_A, "wing")
        .unwrap();
    graph
        .append_partition_detail_dependency(detail, source, ASPECT_A, "wing", "rib-12")
        .unwrap();
    evaluate(&mut graph, whole, &mut |_id, _graph| Ok(version_ab(10, 0))).unwrap();
    evaluate(&mut graph, detail, &mut |_id, _graph| Ok(version_ab(20, 0))).unwrap();

    mark_dirty_with_regions(
        &mut graph,
        source,
        ASPECT_A,
        &[crate::data::output::ChangedRegion::new("wing")],
    )
    .unwrap();
    assert_eq!(graph.get_state(whole).unwrap(), NodeState::Clean);
    assert_eq!(graph.get_state(detail).unwrap(), NodeState::Clean);

    evaluate(&mut graph, source, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(2, 0))
            .with_changed_region(crate::data::output::ChangedRegion::new("wing")))
    })
    .unwrap();

    assert_eq!(graph.get_state(whole).unwrap(), NodeState::Dirty);
    assert_eq!(graph.get_state(detail).unwrap(), NodeState::Dirty);
    assert_eq!(graph.pending_causes(whole).unwrap().len(), 1);
    assert_eq!(graph.pending_causes(detail).unwrap().len(), 1);
}

#[test]
fn source_seed_planning_estimate_does_not_walk_diamond_reachability() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::development());
    let source = graph.node().build();
    evaluate(&mut graph, source, &mut |_id, _graph| Ok(version_ab(1, 0))).unwrap();
    let left = graph.node().build();
    let right = graph.node().build();
    let downstream = graph.node().build();

    graph.append_dependency(left, source, ASPECT_A).unwrap();
    graph.append_dependency(right, source, ASPECT_A).unwrap();
    graph.append_dependency(downstream, left, ASPECT_A).unwrap();
    graph
        .append_dependency(downstream, right, ASPECT_A)
        .unwrap();
    for node in [left, right, downstream] {
        evaluate(&mut graph, node, &mut |_id, _graph| Ok(version_ab(10, 0))).unwrap();
    }

    mark_dirty(&mut graph, source, ASPECT_A).unwrap();

    let estimate = graph
        .observe()
        .latest_invalidation_planning_estimate()
        .expect("source seed should retain its planning estimate");
    assert_eq!(estimate.seed_count(), 1);
    assert_eq!(estimate.direct_candidate_count(), 0);
    assert!(graph.get_state(left).unwrap() == NodeState::Clean);
    assert!(graph.get_state(right).unwrap() == NodeState::Clean);
    assert!(graph.get_state(downstream).unwrap() == NodeState::Clean);
}

#[test]
fn runtime_metrics_surface_typed_reuse_family_counters() {
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
    let compute_calls = std::sync::atomic::AtomicU32::new(0);
    let projection = runtime
        .define(Recipe {
            family: "projection".into(),
            contract: NodeContract::reads([ASPECT_A])
                .with_produces([ASPECT_B])
                .with_cross_identity_persistent_matching()
                .with_partial_artifact_splicing(),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                compute_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("typed-reuse")
                        .with_output_change(OutputChange::Refreshed),
                ))
            },
        })
        .unwrap();
    let source = projection.keyed("source");
    let alias = projection.keyed("alias");
    let splice = projection.keyed("splice");
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            source.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity(tx, "source", "shape-v1", "mesh-telemetry")
        })
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            splice.evaluate_partial_splice(
                tx,
                "shape-v1",
                [PartitionSubscription::whole_partition("wing")],
            )
        })
        .unwrap();

    let metrics = runtime.observe().metrics();
    assert_eq!(metrics.evaluation.memoization_hits, 1);
    assert_eq!(metrics.evaluation.cross_identity_reuse_count, 1);
    assert_eq!(metrics.evaluation.partial_artifact_splice_count, 1);
}

#[test]
fn typed_rejection_counters_match_runtime_reuse_failures() {
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
            family: "restricted-projection".into(),
            contract: NodeContract::reads([ASPECT_A])
                .with_produces([ASPECT_B])
                .with_partition_scope(PartitionSubscription::whole_partition("wing"))
                .with_reuse_contract(NodeReuseContract {
                    equivalence: ArtifactEquivalenceContract {
                        required_boundaries: vec![
                            ArtifactSemanticBoundary::TopologyRegime,
                            ArtifactSemanticBoundary::ToleranceRegime,
                            ArtifactSemanticBoundary::SemanticRegionIdentity,
                            ArtifactSemanticBoundary::ArtifactFamilyBasis,
                            ArtifactSemanticBoundary::StructuralDependencyBasis,
                            ArtifactSemanticBoundary::PartitionRegionBasis,
                        ],
                        supported_strategies: vec![ReuseStrategy::MemoizedArtifactReuse],
                        allows_snapshot_restore_reuse: false,
                        allows_authority_reconciliation_reuse: false,
                    },
                    retain_certification: true,
                }),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("restricted-artifact")
                        .with_output_change(OutputChange::Refreshed)
                        .with_changed_region(ChangedRegion::new("wing")),
                ))
            },
        })
        .unwrap();
    let source = projection.keyed("source");
    let alias = projection.keyed("alias");
    let wing = projection.keyed("wing");
    let wing_node = wing.node(&mut runtime);
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            source.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            wing.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    mark_dirty(runtime.graph_mut(), wing_node, ASPECT_A).unwrap();

    let cross_identity_err = runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity(tx, "source", "shape-v1", "mesh-reject")
        })
        .expect_err("cross-identity should be rejected by the reuse contract");
    let partial_splice_err = runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            wing.evaluate_partial_splice(
                tx,
                "shape-v1",
                [PartitionSubscription::whole_partition("wing")],
            )
        })
        .expect_err("partial splice should be rejected by the reuse contract");

    assert!(cross_identity_err
        .to_string()
        .contains("reuse certification failed"));
    assert!(partial_splice_err
        .to_string()
        .contains("reuse certification failed"));

    let metrics = runtime.observe().metrics();
    assert_eq!(metrics.evaluation.reuse_rejected_contract_strategy_count, 2);
    assert_eq!(metrics.evaluation.cross_identity_reuse_count, 0);
    assert_eq!(metrics.evaluation.partial_artifact_splice_count, 0);
}
