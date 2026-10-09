use crate::facade::*;
use crate::tests::support::*;

#[test]
fn diagnostics_profiles_control_retention_bounds() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let compute = |ctx: &mut EvaluationContext<'_, ()>| Ok(ctx.finish(version_ab(1, 0)));

    graph.reset_runtime_policy_to_tier(DiagnosticsTier::Operational);
    for _ in 0..8 {
        mark_dirty(&mut graph, node, ASPECT_A).unwrap();
        let plan = graph
            .build_evaluation_plan(&[node], EvaluationRequestMode::Default)
            .unwrap();
        graph.execute_prepared_plan(&plan, &(), &compute).unwrap();
    }
    assert!(graph.observe().recent_execution_history_diagnostics().len() <= 4);

    graph.reset_runtime_policy_to_tier(DiagnosticsTier::Forensic);
    for _ in 0..8 {
        mark_dirty(&mut graph, node, ASPECT_A).unwrap();
        let plan = graph
            .build_evaluation_plan(&[node], EvaluationRequestMode::Default)
            .unwrap();
        graph.execute_prepared_plan(&plan, &(), &compute).unwrap();
    }
    assert!(graph.observe().recent_execution_history_diagnostics().len() > 4);
}

#[test]
fn operational_profile_repeated_waves_stay_bounded_and_shallow() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    graph.reset_runtime_policy_to_tier(DiagnosticsTier::Operational);
    let compute = |ctx: &mut EvaluationContext<'_, ()>| Ok(ctx.finish(version_ab(1, 0)));

    for _ in 0..100 {
        mark_dirty(&mut graph, node, ASPECT_A).unwrap();
        let plan = graph
            .build_evaluation_plan(&[node], EvaluationRequestMode::Default)
            .unwrap();
        graph.execute_prepared_plan(&plan, &(), &compute).unwrap();
    }

    let diagnostics = graph.observe().diagnostics();
    let history = diagnostics.recent_history();
    let policy = SignalRuntimePolicy::for_tier(DiagnosticsTier::Operational);
    assert!(history.len() <= policy.retention_budget.history_limit);
    assert!(history.iter().all(|summary| summary.nodes.is_empty()));
    assert!(diagnostics.latest_failure().is_none());
}

#[test]
fn repeated_failure_capture_stays_current_and_bounded() {
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
    let node = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    for cycle in 0..100 {
        let err = runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.mark_dirty(node, ASPECT_A)?;
                tx.evaluate_with_plan(
                    node,
                    &move |_view| {
                        Err::<crate::logic::evaluation::EvaluationOutput, _>(SignalError::internal(
                            format!("synthetic precompute failure cycle {cycle}"),
                        ))
                    },
                    EvaluationRequestMode::Default,
                )?;
                Ok(())
            })
            .unwrap_err();
        assert!(format!("{err}").contains("synthetic precompute failure cycle"));
    }

    let diagnostics = runtime.observe().diagnostics();
    let failure = diagnostics
        .latest_failure()
        .expect("latest failure should be retained");
    assert_eq!(failure.phase, ExecutionFailurePhase::Precompute);
    assert!(failure.message.contains("cycle 99"));
    assert!(
        diagnostics.recent_history().len()
            <= SignalRuntimePolicy::for_tier(DiagnosticsTier::Development)
                .retention_budget
                .history_limit
    );
}

#[test]
fn repeated_memoized_execution_retains_bounded_diagnostics() {
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
    runtime.graph_mut().set_runtime_policy(
        SignalRuntimePolicy::for_tier(DiagnosticsTier::Operational).with_observation_activation(
            worth_foundational::ObservationActivationProfile::Continuous,
        ),
    );
    let family = define_keyed_computation(&mut runtime, "projection", ());
    let keyed = family.keyed("bulkhead");
    let node = keyed.node(&mut runtime);
    let computation = keyed.memoized("shape-v1");
    let mut runtime_ctx = ();

    for cycle in 0..50 {
        let compute_value = cycle as u64 + 1;
        runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.evaluate_keyed(node, &computation, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(compute_value, 0))
                            .with_output_identity("bulkhead-artifact")
                            .with_output_change(OutputChange::Refreshed),
                    ))
                })?;
                Ok(())
            })
            .unwrap();
        mark_dirty(runtime.graph_mut(), node, ASPECT_A).unwrap();
    }

    let diagnostics = runtime.observe().diagnostics();
    assert!(
        diagnostics.recent_history().len()
            <= SignalRuntimePolicy::for_tier(DiagnosticsTier::Operational)
                .retention_budget
                .history_limit
    );
    assert!(diagnostics
        .recent_history()
        .iter()
        .all(|summary| summary.nodes.is_empty()));
    assert!(runtime.observe().metrics().evaluation.memoization_hits >= 1);
}

#[test]
fn repeated_partition_heavy_invalidation_retains_bounded_diagnostics() {
    let mut graph = SignalGraph::new();
    graph.reset_runtime_policy_to_tier(DiagnosticsTier::Development);
    let source = graph.node().partitioned_output().build();
    let wing = graph.node().build();
    let tail = graph.node().build();
    let mut dependencies = DependencyBatchBuilder::new(&mut graph);
    dependencies
        .append_partition_dependency(wing, source, ASPECT_A, "wing")
        .unwrap()
        .append_partition_dependency(tail, source, ASPECT_A, "tail")
        .unwrap();
    dependencies.commit().unwrap();

    let evaluator = |ctx: &mut EvaluationContext<'_, ()>| {
        let result = if ctx.node() == source {
            NodeEvaluationResult::from_version(version_ab(1, 0))
                .with_changed_region(ChangedRegion::new("wing"))
        } else {
            NodeEvaluationResult::from_version(ctx.read_aspect_version(source, ASPECT_A)?)
        };
        Ok(ctx.finish(result))
    };

    let bootstrap = graph
        .build_evaluation_plan(&[source, wing, tail], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &evaluator)
        .unwrap();

    for _ in 0..50 {
        mark_dirty_with_regions(&mut graph, source, ASPECT_A, &[ChangedRegion::new("wing")])
            .unwrap();
        let plan = graph
            .build_evaluation_plan(&[wing, tail], EvaluationRequestMode::Default)
            .unwrap();
        graph.execute_prepared_plan(&plan, &(), &evaluator).unwrap();
    }

    let diagnostics = graph.observe().diagnostics();
    let flow = diagnostics
        .latest_flow()
        .expect("flow diagnostics should be retained");
    let frontier = diagnostics
        .latest_frontier_execution()
        .expect("frontier execution summary should be retained");
    assert_eq!(flow.change.changed_nodes, vec![source]);
    assert_eq!(flow.change.changed_region_count, 1);
    assert_eq!(
        flow.invalidation.invalidated_direct_subscribers
            + flow.invalidation.maybe_stale_direct_subscribers,
        0
    );
    assert_eq!(
        flow.invalidation.frontier_seed_count as u64,
        frontier.counters.frontier_seed_count
    );
    assert_eq!(
        flow.invalidation.frontier_trace_retained_count as usize,
        diagnostics.latest_invalidation_trace_records().len()
    );
    assert!(
        diagnostics.recent_history().len()
            <= SignalRuntimePolicy::for_tier(DiagnosticsTier::Development)
                .retention_budget
                .history_limit
    );
}

#[test]
fn runtime_policy_history_budget_overrides_are_enforced() {
    let policy = SignalRuntimePolicy::development()
        .with_history_limit(2)
        .with_detail_limit(1)
        .with_history_details(true);

    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(policy);
    let source = graph.node().output_identity().build();

    for version in 0..5 {
        mark_dirty(&mut graph, source, ASPECT_A).unwrap_or(());
        evaluate(&mut graph, source, &mut |_id, _graph| {
            Ok(
                NodeEvaluationResult::from_version(version_ab(version + 1, 0))
                    .with_output_identity(format!("budget-{version}")),
            )
        })
        .unwrap();
    }

    let diagnostics = graph.observe().diagnostics();
    assert!(diagnostics.recent_history().len() <= 2);
    assert!(
        diagnostics
            .recent_history()
            .iter()
            .all(|summary| summary.nodes.len() <= 1),
        "detail limit override should trim retained node detail"
    );
}
