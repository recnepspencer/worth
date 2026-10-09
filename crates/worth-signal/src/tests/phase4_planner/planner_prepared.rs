use crate::data::trace::assemble_trace_summary;
use crate::facade::{
    BoundedSignalInputs, CheckedEvaluationContext, NodeContract, StageExecutionOutcome,
};
use crate::facade::{EvaluationContext, EvaluationRequestMode, NodeEvaluationResult, SignalGraph};
use crate::tests::leased_execution::support::{authority, request};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn prepared_plan_captures_dependencies_without_manual_graph_wiring() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let dependent = graph.node().build();

    let source_compute = |ctx: &mut EvaluationContext<'_, ()>| Ok(ctx.finish(version_ab(1, 0)));
    let dependent_compute = |ctx: &mut EvaluationContext<'_, ()>| {
        let version = ctx.read_aspect_version(source, ASPECT_A)?;
        Ok(ctx.finish(NodeEvaluationResult::from_version(version)))
    };

    let source_plan = graph
        .build_evaluation_plan(&[source], EvaluationRequestMode::Default)
        .unwrap();
    graph
        .execute_prepared_plan(&source_plan, &(), &source_compute)
        .unwrap();

    let dependent_plan = graph
        .build_evaluation_plan(&[dependent], EvaluationRequestMode::Default)
        .unwrap();
    graph
        .execute_prepared_plan(&dependent_plan, &(), &dependent_compute)
        .unwrap();

    let dependencies = graph.dependencies_of(dependent).unwrap();
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].source(), source);
    assert_eq!(dependencies[0].aspect(), ASPECT_A);
}

#[test]
fn prepared_parallel_precompute_matches_serial_results() {
    let serial_lease = authority().request_lease(request(1, 10_000_000)).unwrap();
    let parallel_lease = authority().request_lease(request(4, 10_000_000)).unwrap();
    let mut serial_graph = SignalGraph::new();
    let a = serial_graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    let b = serial_graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();

    let mut parallel_graph = serial_graph.clone();
    let parallel_a = a;
    let parallel_b = b;

    let plan = serial_graph
        .build_evaluation_plan(&[a, b], EvaluationRequestMode::Default)
        .unwrap();
    let parallel_plan = parallel_graph
        .build_evaluation_plan(&[parallel_a, parallel_b], EvaluationRequestMode::Default)
        .unwrap();

    let evaluator =
        |ctx: &mut CheckedEvaluationContext<'_, '_, '_, '_, ()>| Ok(ctx.finish(version_ab(7, 0)));

    let serial_report = serial_graph
        .execute_prepared_plan_checked(&plan, &(), &evaluator, &serial_lease)
        .unwrap();
    let parallel_report = parallel_graph
        .execute_prepared_plan_checked(&parallel_plan, &(), &evaluator, &parallel_lease)
        .unwrap();

    assert_eq!(
        serial_graph.get_state(a).unwrap(),
        parallel_graph.get_state(parallel_a).unwrap()
    );
    assert_eq!(
        serial_graph.get_state(b).unwrap(),
        parallel_graph.get_state(parallel_b).unwrap()
    );
    assert_eq!(
        assemble_trace_summary(
            serial_graph
                .get_entry(a)
                .unwrap()
                .get_runtime_artifact_state(),
            serial_graph
                .get_entry(a)
                .unwrap()
                .retained_diagnostic_artifact(),
        )
        .unwrap()
        .output_hash,
        assemble_trace_summary(
            parallel_graph
                .get_entry(parallel_a)
                .unwrap()
                .get_runtime_artifact_state(),
            parallel_graph
                .get_entry(parallel_a)
                .unwrap()
                .retained_diagnostic_artifact(),
        )
        .unwrap()
        .output_hash
    );
    assert_eq!(serial_report.task_count, parallel_report.task_count);
    assert_eq!(serial_report.tasks_executed, parallel_report.tasks_executed);
}

#[test]
fn build_evaluation_plan_handles_deep_linear_chain_without_recursion() {
    let mut graph = SignalGraph::new();
    let root = graph.node().build();
    let mut previous = root;
    let depth = 2_048;

    for _ in 0..depth {
        let current = graph.node().build();
        graph
            .append_dependency(current, previous, ASPECT_A)
            .unwrap();
        previous = current;
    }

    let plan = graph
        .build_evaluation_plan(&[previous], EvaluationRequestMode::Default)
        .unwrap();

    assert_eq!(plan.summary.stage_count, (depth + 1) as u32);
    assert_eq!(plan.stages.first().unwrap().tasks[0].node, root);
    assert_eq!(plan.stages.last().unwrap().tasks[0].node, previous);
}

#[test]
fn parallel_executor_threshold_keeps_narrow_stage_serial() {
    let parallel_lease = authority().request_lease(request(4, 10_000_000)).unwrap();
    let mut graph = SignalGraph::new();
    let node = graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    let plan = graph
        .build_evaluation_plan(&[node], EvaluationRequestMode::Default)
        .unwrap();
    let evaluator =
        |ctx: &mut CheckedEvaluationContext<'_, '_, '_, '_, ()>| Ok(ctx.finish(version_ab(1, 0)));

    let report = graph
        .execute_prepared_plan_checked(&plan, &(), &evaluator, &parallel_lease)
        .unwrap();

    assert!(matches!(
        report.stages[0].outcome,
        StageExecutionOutcome::CompletedSerial
    ));
}

#[test]
fn checked_wide_epoch_preserves_serial_results_and_reports_resolved_placement() {
    if crate::tests::leased_execution::support::private_authority_process::run_with_private_authority(
        "tests::phase4_planner::planner_prepared::checked_wide_epoch_preserves_serial_results_and_reports_resolved_placement",
    ) {
        return;
    }
    let serial_lease = authority().request_lease(request(1, 10_000_000)).unwrap();
    let parallel_lease = authority().request_lease(request(4, 10_000_000)).unwrap();
    let mut serial_graph = SignalGraph::new();
    let serial_nodes = (0..12)
        .map(|_| {
            serial_graph
                .node()
                .with_contract(
                    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
                )
                .build()
        })
        .collect::<Vec<_>>();

    let mut parallel_graph = serial_graph.clone();
    let parallel_nodes = serial_nodes.clone();

    let plan = serial_graph
        .build_evaluation_plan(&serial_nodes, EvaluationRequestMode::Default)
        .unwrap();
    let parallel_plan = parallel_graph
        .build_evaluation_plan(&parallel_nodes, EvaluationRequestMode::Default)
        .unwrap();

    let evaluator =
        |ctx: &mut CheckedEvaluationContext<'_, '_, '_, '_, ()>| Ok(ctx.finish(version_ab(11, 0)));

    let serial_report = serial_graph
        .execute_prepared_plan_checked(&plan, &(), &evaluator, &serial_lease)
        .unwrap();
    let rendezvous =
        crate::tests::leased_execution::support::task_rendezvous::TaskRendezvous::default();
    let parallel_evaluator = |ctx: &mut CheckedEvaluationContext<'_, '_, '_, '_, ()>| {
        rendezvous.meet();
        evaluator(ctx)
    };
    let parallel_report = parallel_graph
        .execute_prepared_plan_checked(&parallel_plan, &(), &parallel_evaluator, &parallel_lease)
        .unwrap();

    for (serial_node, parallel_node) in serial_nodes.iter().zip(parallel_nodes.iter()) {
        assert_eq!(
            serial_graph.get_state(*serial_node).unwrap(),
            parallel_graph.get_state(*parallel_node).unwrap()
        );
    }
    assert_eq!(serial_report.task_count, parallel_report.task_count);
    assert_eq!(serial_report.tasks_executed, parallel_report.tasks_executed);
    let resolved_parallel = parallel_report.execution.iter().any(|execution| {
        execution.resolved_posture() == worth_foundational::ExecutionPosture::Automatic
    });
    // A private process authority has four workers; twelve tasks exceed Balanced/full-apply thresholds four/eight.
    assert!(resolved_parallel);
    if resolved_parallel {
        assert!(
            parallel_graph
                .telemetry()
                .execution
                .parallel_stage_dispatch_count
                > 0
        );
    }
    rendezvous.assert_if_parallel(resolved_parallel);
    assert_eq!(
        parallel_report
            .stages
            .iter()
            .any(|stage| stage.outcome == StageExecutionOutcome::CompletedParallel),
        resolved_parallel
    );
    let admitted_workers = parallel_lease.policy().budget().max_workers().get();
    assert!(parallel_report.execution.iter().all(|execution| {
        let workers = execution.physical().active_workers_high_watermark();
        workers >= 1 && workers <= admitted_workers
    }));
}
