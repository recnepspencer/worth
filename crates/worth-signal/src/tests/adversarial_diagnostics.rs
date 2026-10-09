use crate::data::trace::TraceSummary;
use crate::facade::*;
use crate::runtime_policy::SignalRuntimePolicy;
use crate::tests::support::{evaluate, version_ab, ASPECT_A, ASPECT_B};

#[test]
fn operational_profile_stays_bounded_under_snapshot_and_dependency_churn() {
    let mut graph = SignalGraph::new();
    graph.reset_runtime_policy_to_tier(DiagnosticsTier::Operational);
    let source_a = graph.node().build();
    let source_b = graph.node().build();
    let dependent = graph.node().build();
    let observation = graph
        .begin_observation_session(SignalObservationRequest::telemetry())
        .unwrap();

    let bootstrap = |graph: &mut SignalGraph, use_source_b: bool| {
        graph.drop_dependency(dependent, source_a, ASPECT_A).ok();
        graph.drop_dependency(dependent, source_b, ASPECT_A).ok();
        graph
            .append_dependency(
                dependent,
                if use_source_b { source_b } else { source_a },
                ASPECT_A,
            )
            .unwrap();
    };

    for wave in 0..40 {
        let use_source_b = wave % 2 == 1;
        bootstrap(&mut graph, use_source_b);
        let target_source = if use_source_b { source_b } else { source_a };
        mark_dirty(&mut graph, target_source, ASPECT_A).unwrap();
        let plan = graph
            .build_evaluation_plan(&[dependent], EvaluationRequestMode::Default)
            .unwrap();
        graph
            .execute_prepared_plan(&plan, &(), &|ctx| {
                let result = if ctx.node() == source_a {
                    ctx.finish(version_ab(1 + wave as u64, 0))
                } else if ctx.node() == source_b {
                    ctx.finish(version_ab(10 + wave as u64, 0))
                } else {
                    let version = ctx.read_aspect_version(target_source, ASPECT_A)?;
                    ctx.finish(NodeEvaluationResult::from_version(version))
                };
                Ok(result)
            })
            .unwrap();
    }

    graph
        .finish_observation_session(&observation)
        .expect("telemetry observation should include the churn evaluations");
    let diagnostics = graph.observe().diagnostics();
    let policy = SignalRuntimePolicy::for_tier(DiagnosticsTier::Operational);
    assert!(diagnostics.recent_history().len() <= policy.retention_budget.history_limit);
    assert!(diagnostics.latest_failure().is_none());
    assert!(diagnostics.latest_rollback().is_none());
    assert!(diagnostics.latest_flow().is_some());
}

#[test]
fn mixed_partition_heavy_invalidation_keeps_frontier_counters_and_flow_in_sync() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::development());
    let source = graph.node().partitioned_output().build();
    let whole = graph.node().build();
    let detail_a = graph.node().build();
    let detail_b = graph.node().build();

    graph
        .append_partition_dependency(whole, source, ASPECT_A, "wing")
        .unwrap();
    graph
        .append_partition_detail_dependency(detail_a, source, ASPECT_A, "wing", "rib-12")
        .unwrap();
    graph
        .append_partition_detail_dependency(detail_b, source, ASPECT_A, "wing", "rib-13")
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
            &[source, whole, detail_a, detail_b],
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
        .build_evaluation_plan(&[whole, detail_a, detail_b], EvaluationRequestMode::Default)
        .unwrap();
    graph.execute_prepared_plan(&plan, &(), &evaluator).unwrap();

    let diagnostics = graph.observe().diagnostics();
    let flow = diagnostics
        .latest_flow()
        .expect("flow diagnostics should be available");
    let frontier = diagnostics
        .latest_frontier_execution()
        .expect("frontier execution summary should be available");

    assert_eq!(frontier.seed_count, 1);
    assert!(frontier.direct_waves.is_empty());
    assert!(frontier.transitive_waves.is_empty());
    assert_eq!(
        flow.invalidation.frontier_seed_count as u64,
        frontier.counters.frontier_seed_count
    );
    assert_eq!(
        flow.invalidation.frontier_direct_wave_count as u64,
        frontier.counters.frontier_direct_wave_count
    );
    assert_eq!(
        flow.invalidation.frontier_partition_match_count as u64,
        frontier.counters.frontier_partition_match_count
    );
    assert_eq!(
        flow.invalidation.frontier_detail_match_count as u64,
        frontier.counters.frontier_detail_match_count
    );
    assert_eq!(flow.invalidation.invalidated_direct_subscribers, 0);
    assert_eq!(flow.invalidation.maybe_stale_direct_subscribers, 0);
}

#[test]
fn repeated_failure_and_rollback_loops_preserve_explanation_after_churn() {
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
        .reset_runtime_policy_to_tier(DiagnosticsTier::Development);
    let source_a = runtime.graph_mut().node().build();
    let source_b = runtime.graph_mut().node().build();
    let dependent = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.evaluate_with_plan(
                dependent,
                &|view| {
                    let result = if view.node() == source_a {
                        view.finish(version_ab(1, 0))
                    } else if view.node() == source_b {
                        view.finish(version_ab(2, 0))
                    } else {
                        let version = view.read_aspect_version(source_a, ASPECT_A)?;
                        view.finish(NodeEvaluationResult::from_version(version))
                    };
                    Ok(result)
                },
                EvaluationRequestMode::ForceOnDemand,
            )?;
            Ok(())
        })
        .unwrap();

    for wave in 0..16 {
        let err = runtime.transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(if wave % 2 == 0 { source_a } else { source_b }, ASPECT_A)?;
            tx.evaluate_with_plan(
                dependent,
                &|view| {
                    let result = if view.node() == source_a {
                        view.finish(version_ab(1 + wave as u64, 0))
                    } else if view.node() == source_b {
                        view.finish(version_ab(100 + wave as u64, 0))
                    } else if wave % 2 == 0 {
                        let version = view.read_aspect_version(source_b, ASPECT_A)?;
                        view.finish(NodeEvaluationResult::from_version(version))
                    } else {
                        let version = view.read_aspect_version(source_a, ASPECT_A)?;
                        view.finish(NodeEvaluationResult::from_version(version))
                    };
                    Ok(result)
                },
                EvaluationRequestMode::Default,
            )?;
            Err(SignalError::invalid_input("force rollback after churn"))
        });
        assert!(err.is_err());
    }

    let diagnostics = runtime.observe().diagnostics();
    assert!(diagnostics.latest_rollback().is_some());
    let explanation = runtime.observe().explain(dependent).unwrap();
    assert!(!explanation.upstream.is_empty());
}

#[test]
fn repeated_mixed_aspect_churn_keeps_frontier_grouping_bounded() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::development());
    let source = graph.node().build();
    let dep_a = graph.node().build();
    let dep_b = graph.node().build();
    let dep_both = graph.node().build();

    graph.append_dependency(dep_a, source, ASPECT_A).unwrap();
    graph.append_dependency(dep_b, source, ASPECT_B).unwrap();
    graph.append_dependency(dep_both, source, ASPECT_A).unwrap();
    graph.append_dependency(dep_both, source, ASPECT_B).unwrap();

    for wave in 0..24 {
        mark_dirty_batch(
            &mut graph,
            &DirtyBatch::from_sources([(source, ASPECT_A), (source, ASPECT_B)]),
        )
        .unwrap();

        let frontier = graph
            .observe()
            .latest_frontier_execution_summary()
            .cloned()
            .expect("frontier execution summary should be retained");
        assert_eq!(frontier.seed_count, 2);
        assert!(frontier.direct_waves.is_empty());
        assert!(frontier.transitive_waves.is_empty());
        assert_eq!(frontier.counters.frontier_group_count, 0);
        assert_eq!(frontier.counters.frontier_direct_wave_count, 0);
        assert_eq!(frontier.counters.frontier_transitive_wave_count, 0);

        evaluate(&mut graph, source, &mut |_id, _graph| {
            Ok(version_ab(wave as u64 + 1, wave as u64 + 1))
        })
        .unwrap();
        for node in [dep_a, dep_b, dep_both] {
            evaluate(&mut graph, node, &mut |_id, graph| {
                Ok(NodeEvaluationResult::from_version(
                    graph.get_entry(source).unwrap().get_aspect_version(),
                ))
            })
            .unwrap();
        }
    }
}

#[test]
#[ignore = "stress coverage for repeated development-profile diagnostics waves"]
fn stress_development_profile_repeated_waves_remains_semantically_stable() {
    let mut graph = SignalGraph::new();
    graph.reset_runtime_policy_to_tier(DiagnosticsTier::Development);
    let source = graph.node().output_identity().build();
    let dependents: Vec<_> = (0..64)
        .map(|_| graph.node().partitioned_output().build())
        .collect();
    for (index, &dependent) in dependents.iter().enumerate() {
        graph
            .append_partition_detail_dependency(
                dependent,
                source,
                ASPECT_A,
                "wing",
                format!("rib-{index}"),
            )
            .unwrap();
    }

    for wave in 0..200 {
        mark_dirty_with_regions(
            &mut graph,
            source,
            ASPECT_A,
            &[ChangedRegion::new("wing").with_detail(format!("rib-{}", wave % 64))],
        )
        .unwrap();
        let plan = graph
            .build_evaluation_plan(&dependents, EvaluationRequestMode::Default)
            .unwrap();
        graph
            .execute_prepared_plan(&plan, &(), &|ctx| {
                let result = if ctx.node() == source {
                    ctx.finish(
                        NodeEvaluationResult::from_version(version_ab(wave as u64 + 1, 0))
                            .with_output_identity("wing-artifact")
                            .with_changed_region(
                                ChangedRegion::new("wing")
                                    .with_detail(format!("rib-{}", wave % 64)),
                            ),
                    )
                } else {
                    let version = ctx.read_partitioned_aspect_version(
                        source,
                        ASPECT_A,
                        PartitionSubscription::partition_and_detail(
                            "wing",
                            format!("rib-{}", wave % 64),
                        ),
                    )?;
                    ctx.finish(NodeEvaluationResult::from_version(version))
                };
                Ok(result)
            })
            .unwrap();
    }

    assert!(graph.observe().diagnostics().recent_history().len() > 1);
    assert!(graph.observe().latest_failure_diagnostics().is_none());
}

#[test]
fn execution_history_prefers_most_recent_records_over_low_arena_indices() {
    let mut graph = SignalGraph::new();
    graph.reset_runtime_policy_to_tier(DiagnosticsTier::Development);
    let mut nodes = Vec::new();
    for _ in 0..96 {
        nodes.push(graph.node().build());
    }

    for (record_id, &node) in nodes.iter().enumerate() {
        let mut trace = TraceSummary::default();
        trace.execution_record_id = Some(record_id as u64 + 1);
        graph
            .get_entry_mut(node)
            .unwrap()
            .set_trace_summary(Some(trace));
    }

    let history = graph
        .observe()
        .execution_history_summary(DiagnosticsTier::Development);
    let retained = history
        .nodes
        .iter()
        .map(|summary| summary.node)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(
        retained.contains(&nodes[79]),
        "history detail should retain newest high-index node executions instead of truncating by arena order: {history:?}"
    );
    assert!(
        !retained.contains(&nodes[0]),
        "history detail should not be dominated by stale low-index nodes when detail_limit is exceeded: {history:?}"
    );
}
