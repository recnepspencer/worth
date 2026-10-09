use super::schema_registry_scenarios::cross_identity_merge_schema_registry;
use crate::facade::{NodeEvaluationResult, SignalGraph, SignalRuntime};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn proof_minimal_overlap_and_conservative_expansion_remain_distinct_and_bounded() {
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
                    NodeEvaluationResult::from_version(version_ab(13, 0))
                        .with_output_identity("proof-shared"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime.create_branch("feature-proof-overlap").unwrap();
    runtime.switch_branch(feature.clone()).unwrap();

    let source_only = runtime.graph_mut().node().output_identity().build();
    runtime
        .graph_mut()
        .append_dependency(source_only, shared, ASPECT_A)
        .unwrap();

    runtime.switch_branch(main).unwrap();
    let result = runtime
        .merge_branch_raw(feature, runtime.observe().current_branch())
        .unwrap();

    assert!(
        result.proof_minimal_overlap.shared_nodes.is_empty(),
        "source-only introductions should not fabricate shared-node minimal overlap"
    );
    assert_eq!(
        result.planned_candidates.nodes,
        vec![source_only],
        "final merge candidates should remain bounded to the source mutation set"
    );
    assert!(
        result.conservative_overlap.support_nodes.contains(&shared),
        "conservative overlap should expand only to the target support surface needed for remaps"
    );
    assert!(
        result.counters.proof_minimal_overlap_breadth
            < result.counters.conservative_overlap_expansion_breadth,
        "conservative expansion should be measurably broader than proof-minimal overlap here"
    );
    assert!(
        result.counters.final_candidate_breadth
            < result.counters.conservative_overlap_expansion_breadth,
        "support expansion should remain distinct from the final candidate set"
    );
}

#[test]
fn merge_candidate_construction_is_identical_with_and_without_convenience_branch_indexes() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let prepare_runtime = || {
        let mut runtime = SignalRuntime::builder(SignalGraph::new())
            .with_kernel_defaults()
            .build();
        let shared = runtime.graph_mut().node().output_identity().build();
        let mut runtime_ctx = ();

        runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.read(shared, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(14, 0))
                            .with_output_identity("index-shared"),
                    ))
                })?;
                Ok(())
            })
            .unwrap();

        let main = runtime.observe().current_branch();
        let feature = runtime.create_branch("feature-index-stability").unwrap();
        runtime.switch_branch(feature.clone()).unwrap();

        let source_only = runtime.graph_mut().node().output_identity().build();
        runtime
            .graph_mut()
            .append_dependency(source_only, shared, ASPECT_A)
            .unwrap();

        runtime.switch_branch(main.clone()).unwrap();
        (runtime, feature, main)
    };

    let (mut baseline_runtime, baseline_feature, baseline_main) = prepare_runtime();
    let initial = baseline_runtime
        .inspect_branch_merge_plan_for_test(baseline_feature, baseline_main)
        .unwrap();

    let (mut rebuilt_runtime, rebuilt_feature, rebuilt_main) = prepare_runtime();
    rebuilt_runtime
        .graph_mut()
        .rebuild_subscriber_index_from_dependencies()
        .unwrap();
    let rebuilt = rebuilt_runtime
        .inspect_branch_merge_plan_for_test(rebuilt_feature, rebuilt_main)
        .unwrap();

    assert_eq!(
        initial.planned_candidates(),
        rebuilt.planned_candidates(),
        "convenience index rebuilds must not change planned merge candidates"
    );
    assert_eq!(
        initial.proof_minimal_overlap(),
        rebuilt.proof_minimal_overlap(),
        "convenience index rebuilds must not widen proof-minimal overlap"
    );
    assert_eq!(
        initial.conservative_overlap(),
        rebuilt.conservative_overlap(),
        "convenience index rebuilds must not change bounded conservative expansion"
    );
    assert_eq!(
        initial.target_overlap_journal(),
        rebuilt.target_overlap_journal(),
        "convenience index rebuilds must not perturb target overlap classification"
    );
}

#[test]
fn merge_budget_identity_counters_track_bounded_target_journal_scope() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(cross_identity_merge_schema_registry(None));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let mut runtime_ctx = ();
    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-identity-budget-counters")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    let feature_only = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.cross-identity-merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(feature_only, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(34, 0))
                        .with_output_identity("identity-budget-shared"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    let matched_target = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.cross-identity-merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    let unrelated_target = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.cross-identity-merge-owned")
        .expect("known schema")
        .output_identity()
        .build();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(matched_target, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(35, 0))
                        .with_output_identity("identity-budget-shared"),
                ))
            })?;
            tx.read(unrelated_target, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(36, 0))
                        .with_output_identity("identity-budget-unrelated"),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    let result = runtime
        .merge_raw()
        .from(feature)
        .into(main)
        .identity_matcher_named("signal.identity.output-identity-in-target-journal")
        .run()
        .unwrap();

    assert_eq!(result.identity_correspondence.target_candidate_count, 2);
    assert_eq!(result.counters.identity_target_candidates_indexed, 2);
    assert_eq!(result.identity_correspondence.source_lookup_count, 1);
    assert_eq!(result.counters.identity_source_lookups, 1);
    assert_eq!(result.identity_correspondence.ambiguous_match_count, 0);
    assert_eq!(result.counters.identity_ambiguous_match_count, 0);
    assert_eq!(
        result.identity_correspondence.rejected_admissibility_count,
        0
    );
    assert_eq!(result.counters.identity_rejected_admissibility_count, 0);
    let correspondence = result
        .identity_correspondence
        .records
        .iter()
        .find(|record| record.source_node == feature_only)
        .expect("identity correspondence record should be present");
    assert_eq!(correspondence.target_node, Some(matched_target));
    assert_eq!(correspondence.candidate_count, 1);
}
