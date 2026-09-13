use crate::facade::*;
use crate::tests::support::*;
use std::sync::atomic::{AtomicU32, Ordering};

#[test]
fn graph_diagnostics_summary_is_deterministic_and_serializable() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let dependent = graph.node().build();
    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();

    let mut source_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    let mut dependent_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(10, 0));
    evaluate(&mut graph, source, &mut source_compute).unwrap();
    evaluate(&mut graph, dependent, &mut dependent_compute).unwrap();

    let left = graph
        .observe()
        .diagnostics_summary(DiagnosticsTier::Development);
    let right = graph
        .observe()
        .diagnostics_summary(DiagnosticsTier::Development);
    assert_eq!(left, right);
    assert!(graphs_semantically_equivalent(&left, &right));

    let json = serde_json::to_string(&left).unwrap();
    let restored: GraphSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(left, restored);
    assert!(render_graph_summary(&left).contains("GraphSummary"));
}

#[test]
fn history_and_explanation_summaries_are_deterministic() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let compute = |ctx: &mut EvaluationContext<'_, ()>| Ok(ctx.finish(version_ab(3, 0)));
    let plan = graph
        .build_evaluation_plan(&[node], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph.execute_prepared_plan(&plan, &(), &compute).unwrap();

    let history_a = graph
        .observe()
        .execution_history_summary(DiagnosticsTier::Forensic);
    let history_b = graph
        .observe()
        .execution_history_summary(DiagnosticsTier::Forensic);
    assert!(compare_execution_history(&history_a, &history_b).is_empty());
    assert!(repeat_run_summaries_equal(&[
        history_a.clone(),
        history_b.clone()
    ]));
    assert!(render_execution_history_summary(&history_a).contains("ExecutionHistorySummary"));

    let explanation_a = graph
        .observe()
        .explain(node)
        .unwrap()
        .diagnostics_summary(DiagnosticsTier::Development);
    let explanation_b = graph
        .observe()
        .explain(node)
        .unwrap()
        .diagnostics_summary(DiagnosticsTier::Development);
    assert!(compare_explanations(&explanation_a, &explanation_b).is_empty());
    assert!(explanations_semantically_equivalent(
        &explanation_a,
        &explanation_b
    ));
    assert!(render_explanation_summary(&explanation_a).contains("ExplanationSummary"));
}

#[test]
fn diagnostics_history_and_replay_preserve_typed_advanced_reuse_origins() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .runtime_policy(SignalRuntimePolicy::kernel())
        .build();
    let compute_calls = AtomicU32::new(0);
    let projection = runtime
        .define(Recipe {
            family: "diagnostics-projection".into(),
            contract: NodeContract::reads([ASPECT_A])
                .with_produces([ASPECT_B])
                .with_cross_identity_persistent_matching()
                .with_partial_artifact_splicing()
                .with_partition_scope(PartitionSubscription::whole_partition("wing")),
            tier: (),
            comparator: VersionComparatorPolicy::OutputIdentity,
            evaluator: |view: &mut EvaluationContext<'_, ()>| {
                compute_calls.fetch_add(1, Ordering::Relaxed);
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("diagnostics-artifact")
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
        .transaction(&mut runtime_ctx, |tx| {
            source.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    runtime
        .transaction(&mut runtime_ctx, |tx| {
            alias.evaluate_cross_identity(tx, "source", "shape-v1", "mesh-001")
        })
        .unwrap();
    runtime
        .transaction(&mut runtime_ctx, |tx| {
            wing.evaluate_memoized(tx, "shape-v1")
        })
        .unwrap();
    mark_dirty(runtime.graph_mut(), wing_node, ASPECT_A).unwrap();
    runtime
        .transaction(&mut runtime_ctx, |tx| {
            wing.evaluate_partial_splice(
                tx,
                "shape-v1",
                [PartitionSubscription::whole_partition("wing")],
            )
        })
        .unwrap();

    assert_eq!(compute_calls.load(Ordering::Relaxed), 2);

    let replay = runtime.graph().replay_events();
    assert!(replay.iter().any(|event| {
        event.kind == ReplayEventKind::TaskApplied
            && event.node == Some(alias_node)
            && event.reuse_origin == Some(ReuseOrigin::CrossIdentityPersistentReuse)
    }));
    assert!(replay.iter().any(|event| {
        event.kind == ReplayEventKind::TaskApplied
            && event.node == Some(wing_node)
            && event.reuse_origin == Some(ReuseOrigin::PartialArtifactSplice)
    }));

    let history = runtime
        .observe()
        .execution_history_summary(DiagnosticsTier::Development);
    assert_eq!(
        history
            .reuse_origin_counts
            .get(&ReuseOrigin::CrossIdentityPersistentReuse)
            .copied(),
        Some(1)
    );
    assert_eq!(
        history
            .reuse_origin_counts
            .get(&ReuseOrigin::PartialArtifactSplice)
            .copied(),
        Some(1)
    );
    assert!(history.nodes.iter().any(|node| {
        node.node == alias_node
            && node.reuse_origin == Some(ReuseOrigin::CrossIdentityPersistentReuse)
    }));
    assert!(history.nodes.iter().any(|node| {
        node.node == wing_node && node.reuse_origin == Some(ReuseOrigin::PartialArtifactSplice)
    }));

    let recent = runtime.observe().recent_execution_history_diagnostics();
    let latest = recent.last().expect("recent history entry");
    assert_eq!(
        latest
            .reuse_origin_counts
            .get(&ReuseOrigin::CrossIdentityPersistentReuse)
            .copied(),
        Some(1)
    );
    assert_eq!(
        latest
            .reuse_origin_counts
            .get(&ReuseOrigin::PartialArtifactSplice)
            .copied(),
        Some(1)
    );
}

#[test]
fn mixed_direct_and_transitive_frontier_counters_stay_aligned_with_flow_diagnostics() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::development());
    let source = graph.node().partitioned_output().build();
    let direct = graph.node().build();
    let maybe_stale = graph.node().build();
    let transitive = graph.node().build();

    graph
        .append_partition_detail_dependency(direct, source, ASPECT_A, "wing", "rib-12")
        .unwrap();
    graph
        .append_partition_detail_dependency(maybe_stale, source, ASPECT_A, "wing", "rib-13")
        .unwrap();
    graph
        .append_dependency(transitive, direct, ASPECT_A)
        .unwrap();
    graph
        .append_dependency(transitive, maybe_stale, ASPECT_A)
        .unwrap();

    let evaluator = |ctx: &mut EvaluationContext<'_, ()>| {
        let result = if ctx.node() == source {
            ctx.finish(
                NodeEvaluationResult::from_version(version_ab(1, 0))
                    .with_changed_region(ChangedRegion::new("wing").with_detail("rib-12")),
            )
        } else {
            let version = ctx.read_aspect_version(source, ASPECT_A)?;
            ctx.finish(NodeEvaluationResult::from_version(version))
        };
        Ok(result)
    };

    let bootstrap = graph
        .build_evaluation_plan(
            &[source, direct, maybe_stale, transitive],
            EvaluationRequestMode::ForceOnDemand,
        )
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &evaluator)
        .unwrap();

    mark_dirty_with_regions(
        &mut graph,
        source,
        ASPECT_A,
        &[ChangedRegion::new("wing").with_detail("rib-12")],
    )
    .unwrap();

    let plan = graph
        .build_evaluation_plan(
            &[direct, maybe_stale, transitive],
            EvaluationRequestMode::Default,
        )
        .unwrap();
    graph.execute_prepared_plan(&plan, &(), &evaluator).unwrap();

    let diagnostics = graph.observe().diagnostics();
    let flow = diagnostics
        .latest_flow()
        .expect("flow diagnostics should be retained");
    let frontier = diagnostics
        .latest_frontier_execution()
        .expect("frontier execution summary should be retained");

    assert_eq!(
        flow.invalidation.frontier_seed_count as u64,
        frontier.counters.frontier_seed_count
    );
    assert_eq!(
        flow.invalidation.frontier_direct_wave_count as u64,
        frontier.counters.frontier_direct_wave_count
    );
    assert_eq!(
        flow.invalidation.frontier_transitive_wave_count as u64,
        frontier.counters.frontier_transitive_wave_count
    );
    assert_eq!(
        flow.invalidation.frontier_cycle_check_candidate_count as u64,
        frontier.counters.frontier_cycle_check_candidate_count
    );
    assert_eq!(
        flow.invalidation.frontier_cycle_check_visited_count as u64,
        frontier.counters.frontier_cycle_check_visited_count
    );
    assert_eq!(
        flow.invalidation.invalidated_direct_subscribers,
        frontier
            .direct_waves
            .iter()
            .flat_map(|wave| wave.entries.iter())
            .filter(|entry| matches!(
                entry.classification,
                FrontierEntryClassification::DirectDirty
            ))
            .count() as u32
    );
    assert_eq!(
        flow.invalidation.maybe_stale_direct_subscribers,
        frontier
            .direct_waves
            .iter()
            .flat_map(|wave| wave.entries.iter())
            .filter(|entry| matches!(
                entry.classification,
                FrontierEntryClassification::MaybeStale
            ))
            .count() as u32
    );
}

#[test]
fn flow_diagnostics_report_zero_realized_transitive_waves_when_frontier_has_none() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::development());
    let source = graph.node().partitioned_output().build();
    let direct = graph.node().build();

    graph
        .append_partition_detail_dependency(direct, source, ASPECT_A, "wing", "rib-12")
        .unwrap();

    let evaluator = |ctx: &mut EvaluationContext<'_, ()>| {
        let result = if ctx.node() == source {
            ctx.finish(
                NodeEvaluationResult::from_version(version_ab(1, 0))
                    .with_changed_region(ChangedRegion::new("wing").with_detail("rib-12")),
            )
        } else {
            let version = ctx.read_aspect_version(source, ASPECT_A)?;
            ctx.finish(NodeEvaluationResult::from_version(version))
        };
        Ok(result)
    };

    let bootstrap = graph
        .build_evaluation_plan(&[source, direct], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &evaluator)
        .unwrap();

    mark_dirty_with_regions(
        &mut graph,
        source,
        ASPECT_A,
        &[ChangedRegion::new("wing").with_detail("rib-12")],
    )
    .unwrap();

    let plan = graph
        .build_evaluation_plan(&[direct], EvaluationRequestMode::Default)
        .unwrap();
    graph.execute_prepared_plan(&plan, &(), &evaluator).unwrap();

    let diagnostics = graph.observe().diagnostics();
    let flow = diagnostics
        .latest_flow()
        .expect("flow diagnostics should be retained");
    let frontier = diagnostics
        .latest_frontier_execution()
        .expect("frontier execution summary should be retained");

    assert!(frontier
        .transitive_waves
        .iter()
        .all(|wave| wave.entries.is_empty()));
    assert_eq!(frontier.counters.frontier_transitive_wave_count, 0);
    assert_eq!(flow.invalidation.frontier_transitive_wave_count, 0);
}
