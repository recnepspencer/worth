use crate::facade::*;
use crate::tests::leased_execution::support::{authority, request};
use crate::tests::support::*;

#[test]
fn graph_diff_detects_state_and_structure_mismatch() {
    let mut graph_a = SignalGraph::new();
    let node_a = graph_a.node().build();
    let mut compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    evaluate(&mut graph_a, node_a, &mut compute).unwrap();

    let mut graph_b = SignalGraph::new();
    let source = graph_b.node().build();
    let dependent = graph_b.node().build();
    graph_b
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();
    evaluate(&mut graph_b, source, &mut compute).unwrap();
    evaluate(&mut graph_b, dependent, &mut compute).unwrap();
    mark_dirty(&mut graph_b, source, ASPECT_A).unwrap();

    let left = graph_a.diagnostics_summary(DiagnosticsTier::Operational);
    let right = graph_b.diagnostics_summary(DiagnosticsTier::Operational);
    let diff = compare_graphs(&left, &right);
    assert!(!diff.is_empty());
    assert!(!graphs_semantically_equivalent(&left, &right));
}

#[test]
fn flow_and_failure_summaries_are_structured_and_diffable() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    graph
        .set_causality(
            node,
            Some(CausalityMetadata {
                kind: "host-change".to_string(),
                fields: Default::default(),
            }),
        )
        .unwrap();

    let compute = |ctx: &mut EvaluationContext<'_, ()>| Ok(ctx.finish(version_ab(2, 0)));
    let plan = graph
        .build_evaluation_plan(&[node], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    let report = graph.execute_prepared_plan(&plan, &(), &compute).unwrap();
    let explanation = graph.observe().explain(node).unwrap();

    let flow = FlowSummary::new(
        DiagnosticsTier::Development,
        ChangeInputSummary::new(
            vec![node],
            vec![ASPECT_A],
            0,
            Some("host-change".to_string()),
        ),
        InvalidationSummary::new(1, 0, 0, 0, 0),
        PlanningSummary::from_plan(&plan, DiagnosticsTier::Development),
        PrecomputeSummary::from_report(&report, DiagnosticsTier::Development),
        ApplySummary::from_report(&report, DiagnosticsTier::Development),
        Vec::new(),
        Vec::new(),
        None,
        None,
        Some(explanation.diagnostics_summary(DiagnosticsTier::Development)),
    );
    let flow_clone = flow.clone();
    assert!(compare_flows(&flow, &flow_clone).is_empty());
    assert_eq!(inspect_flow(&flow).changed_nodes(), &[node]);
    assert!(render_flow_summary(&flow).contains("FlowSummary"));

    let failure = ExecutionFailureContext::new(
        ExecutionFailurePhase::Precompute,
        Some(0),
        Some(node),
        Some(worth_foundational::ExecutionPosture::Serial),
        Some(ExecutionRecordId(7)),
        Some(plan.summary),
        "precompute failed",
    );
    let rollback = RollbackDiagnostic::new(
        true,
        3,
        2,
        Some("rewound staged changes".to_string()),
        Vec::new(),
    );
    let failure_summary = failure.summarize(Some(&rollback), DiagnosticsTier::Forensic);
    let failure_summary_2 = failure.summarize(None, DiagnosticsTier::Forensic);
    assert!(!compare_failures(&failure_summary, &failure_summary_2).is_empty());
    assert!(render_failure_summary(&failure_summary).contains("FailureSummary"));
}

#[test]
fn serial_and_parallel_reports_are_semantically_equivalent() {
    let mut graph_serial = SignalGraph::new();
    let a = graph_serial
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    let b = graph_serial
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    let c = graph_serial
        .node()
        .with_contract(
            NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new([
                DeclaredSignalInput::new(a, ASPECT_A),
                DeclaredSignalInput::new(b, ASPECT_A),
            ])),
        )
        .build();
    let mut dependencies = DependencyBatchBuilder::new(&mut graph_serial);
    dependencies
        .append_dependency(c, a, ASPECT_A)
        .unwrap()
        .append_dependency(c, b, ASPECT_A)
        .unwrap();
    dependencies.commit().unwrap();

    let mut graph_parallel = graph_serial.clone();

    let evaluator = |ctx: &mut EvaluationContext<'_, ()>| {
        let result = if ctx.node() == c {
            let a_v = ctx.read_aspect_version(a, ASPECT_A)?;
            let b_v = ctx.read_aspect_version(b, ASPECT_A)?;
            NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                ASPECT_A,
                a_v.get(ASPECT_A) + b_v.get(ASPECT_A),
            )]))
        } else {
            NodeEvaluationResult::from_version(version_ab(1, 0))
        };
        Ok(ctx.finish(result))
    };

    let bootstrap = graph_serial
        .build_evaluation_plan(&[a, b, c], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph_serial
        .execute_prepared_plan(&bootstrap, &(), &evaluator)
        .unwrap();
    graph_parallel
        .execute_prepared_plan(&bootstrap, &(), &evaluator)
        .unwrap();

    let checked_evaluator = |ctx: &mut CheckedEvaluationContext<'_, '_, '_, '_, ()>| {
        let value = if ctx.node() == c {
            ctx.read(a, ASPECT_A)? + ctx.read(b, ASPECT_A)?
        } else {
            1
        };
        Ok(version_ab(value, 0))
    };
    let serial_lease = authority().request_lease(request(1, 1_000_000)).unwrap();
    let parallel_lease = authority().request_lease(request(4, 1_000_000)).unwrap();
    mark_dirty(&mut graph_serial, a, ASPECT_A).unwrap();
    mark_dirty(&mut graph_parallel, a, ASPECT_A).unwrap();

    let plan_serial = graph_serial
        .build_evaluation_plan(&[c], EvaluationRequestMode::Default)
        .unwrap();
    let plan_parallel = graph_parallel
        .build_evaluation_plan(&[c], EvaluationRequestMode::Default)
        .unwrap();

    let report_serial = graph_serial
        .execute_prepared_plan_checked(
            &plan_serial,
            &(),
            &checked_evaluator,
            worth_execution::ExecutionRequest::leased(&serial_lease),
        )
        .unwrap();
    let report_parallel = graph_parallel
        .execute_prepared_plan_checked(
            &plan_parallel,
            &(),
            &checked_evaluator,
            worth_execution::ExecutionRequest::leased(&parallel_lease),
        )
        .unwrap();

    let summary_serial = report_serial.diagnostics_summary(DiagnosticsTier::Development);
    let summary_parallel = report_parallel.diagnostics_summary(DiagnosticsTier::Development);
    assert!(serial_parallel_reports_equivalent(
        &summary_serial,
        &summary_parallel
    ));
}

#[test]
fn repeated_serial_parallel_lifecycle_parity_stays_stable() {
    let mut graph_serial = SignalGraph::new();
    let source = graph_serial
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    let wing = graph_serial
        .node()
        .with_contract(
            NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new([
                DeclaredSignalInput::new(source, ASPECT_A),
            ])),
        )
        .build();
    let tail = graph_serial
        .node()
        .with_contract(
            NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new([
                DeclaredSignalInput::new(source, ASPECT_A),
            ])),
        )
        .build();
    let mut dependencies = DependencyBatchBuilder::new(&mut graph_serial);
    dependencies
        .append_partition_dependency(wing, source, ASPECT_A, "wing")
        .unwrap()
        .append_partition_dependency(tail, source, ASPECT_A, "tail")
        .unwrap();
    dependencies.commit().unwrap();
    let mut graph_parallel = graph_serial.clone();

    let evaluator = |ctx: &mut EvaluationContext<'_, ()>| {
        let result = if ctx.node() == source {
            NodeEvaluationResult::from_version(version_ab(1, 0))
                .with_changed_region(ChangedRegion::new("wing"))
        } else {
            let version = ctx.read_aspect_version(source, ASPECT_A)?;
            NodeEvaluationResult::from_version(version)
        };
        Ok(ctx.finish(result))
    };

    let bootstrap = graph_serial
        .build_evaluation_plan(&[source, wing, tail], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph_serial
        .execute_prepared_plan(&bootstrap, &(), &evaluator)
        .unwrap();
    graph_parallel
        .execute_prepared_plan(&bootstrap, &(), &evaluator)
        .unwrap();

    let checked_evaluator = |ctx: &mut CheckedEvaluationContext<'_, '_, '_, '_, ()>| {
        let result = if ctx.node() == source {
            NodeEvaluationResult::from_version(version_ab(1, 0))
                .with_changed_region(ChangedRegion::new("wing"))
        } else {
            NodeEvaluationResult::from_version(version_ab(ctx.read(source, ASPECT_A)?, 0))
        };
        Ok(result)
    };
    let serial_lease = authority().request_lease(request(1, 1_000_000)).unwrap();
    let parallel_lease = authority().request_lease(request(4, 1_000_000)).unwrap();
    for _ in 0..25 {
        mark_dirty_with_regions(
            &mut graph_serial,
            source,
            ASPECT_A,
            &[ChangedRegion::new("wing")],
        )
        .unwrap();
        mark_dirty_with_regions(
            &mut graph_parallel,
            source,
            ASPECT_A,
            &[ChangedRegion::new("wing")],
        )
        .unwrap();

        let plan_serial = graph_serial
            .build_evaluation_plan(&[wing, tail], EvaluationRequestMode::Default)
            .unwrap();
        let plan_parallel = graph_parallel
            .build_evaluation_plan(&[wing, tail], EvaluationRequestMode::Default)
            .unwrap();

        let report_serial = graph_serial
            .execute_prepared_plan_checked(
                &plan_serial,
                &(),
                &checked_evaluator,
                worth_execution::ExecutionRequest::leased(&serial_lease),
            )
            .unwrap();
        let report_parallel = graph_parallel
            .execute_prepared_plan_checked(
                &plan_parallel,
                &(),
                &checked_evaluator,
                worth_execution::ExecutionRequest::leased(&parallel_lease),
            )
            .unwrap();

        assert!(serial_parallel_reports_equivalent(
            &report_serial.diagnostics_summary(DiagnosticsTier::Development),
            &report_parallel.diagnostics_summary(DiagnosticsTier::Development),
        ));
        let serial_flow = graph_serial.observe().latest_flow_diagnostics().unwrap();
        let parallel_flow = graph_parallel.observe().latest_flow_diagnostics().unwrap();
        assert_eq!(serial_flow.change, parallel_flow.change);
        assert_eq!(serial_flow.invalidation, parallel_flow.invalidation);
        assert_eq!(serial_flow.planning, parallel_flow.planning);
        assert!(serial_parallel_reports_equivalent(
            &serial_flow.apply.report,
            &parallel_flow.apply.report,
        ));
        assert_eq!(
            serial_flow.apply.prepared_evaluations_applied,
            parallel_flow.apply.prepared_evaluations_applied
        );
        assert_eq!(
            serial_flow.apply.dependency_capture_updates,
            parallel_flow.apply.dependency_capture_updates
        );
        assert_eq!(
            serial_flow.apply.tasks_validated_clean,
            parallel_flow.apply.tasks_validated_clean
        );
        assert_eq!(
            serial_flow.apply.tasks_pruned,
            parallel_flow.apply.tasks_pruned
        );
        assert_eq!(
            serial_flow.apply.tasks_with_suppressed_propagation,
            parallel_flow.apply.tasks_with_suppressed_propagation
        );
    }
}
