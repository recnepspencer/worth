use super::Tier;
use crate::facade::{
    mark_dirty, DependencyMode, DirtyPropagation, EvaluationOutput, EvaluationRequestMode,
    EvaluationTrigger, NodeId, NodeState, SignalError, SignalGraph, SignalRuntime,
    TaskExecutionOutcome, TierPolicy, VersionComparatorPolicy,
};
use crate::tests::support::{evaluate, version_ab, GraphDependencyBatchExt, ASPECT_A, ASPECT_B};
use std::sync::atomic::{AtomicU32, Ordering};

#[test]
fn runtime_plan_excludes_resolved_irrelevant_aspect_change() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let dependent = graph.node().build();
    let mut source_v10 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(0, 10));
    evaluate(&mut graph, source, &mut source_v10).unwrap();
    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();

    let mut runtime = SignalRuntime::builder(graph)
        .with_kernel_defaults()
        .with_tiers::<Tier>()
        .build();
    runtime.set_node_tier(dependent, Tier::A);
    runtime.set_tier_policy(
        TierPolicy::new(
            Tier::A,
            DependencyMode::AutoDiscovered,
            DirtyPropagation::Immediate,
            EvaluationTrigger::LazyPull,
        )
        .with_default_comparator(VersionComparatorPolicy::Tolerance { epsilon: 2 }),
    );

    let mut source_v12 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(0, 12));
    let mut dependent_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(0, 100));
    evaluate(runtime.graph_mut(), dependent, &mut dependent_compute).unwrap();
    mark_dirty(runtime.graph_mut(), source, ASPECT_B).unwrap();
    evaluate(runtime.graph_mut(), source, &mut source_v12).unwrap();

    let plan = runtime
        .build_evaluation_plan(&[dependent], EvaluationRequestMode::Default)
        .unwrap();

    assert_eq!(
        runtime.graph().get_state(dependent).unwrap(),
        NodeState::Clean
    );
    assert_eq!(plan.summary.task_count, 0);
}

#[test]
fn public_evaluate_routes_through_planner_and_records_execution_metadata() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();

    let mut compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(3, 0));
    evaluate(&mut graph, node, &mut compute).unwrap();

    let trace = graph.get_entry(node).unwrap().trace_summary().unwrap();
    assert!(trace.execution_record_id.is_some());

    let metrics = graph.observe().metrics();
    assert!(metrics.planner.plans_built >= 1);
    assert!(metrics.planner.tasks_scheduled >= 1);
    assert!(metrics.execution.serial_executor_usage_count >= 1);
}

#[test]
fn runtime_execute_plan_with_executor_serial_matches_default_path() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let dependent = graph.node().build();
    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();

    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let mut source_v1 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    let source_v2 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(2, 0));
    let mut dependent_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(10, 0));

    evaluate(runtime.graph_mut(), source, &mut source_v1).unwrap();
    evaluate(runtime.graph_mut(), dependent, &mut dependent_compute).unwrap();
    mark_dirty(runtime.graph_mut(), source, ASPECT_A).unwrap();

    let plan = runtime
        .build_evaluation_plan(&[dependent], EvaluationRequestMode::Default)
        .unwrap();
    let report = runtime
        .execute_prepared_plan(&plan, &(), &|view| {
            let node = view.node();
            if node == source {
                Ok(view.finish(source_v2(node, view.graph())?))
            } else {
                Ok(view.finish(dependent_compute(node, view.graph())?))
            }
        })
        .unwrap();

    assert_eq!(report.stage_count, 2);
    assert_eq!(
        runtime.graph().get_state(dependent).unwrap(),
        NodeState::Clean
    );
}

#[test]
fn execution_report_marks_requested_maybe_stale_validation_as_validated_clean() {
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

    mark_dirty(&mut graph, source, ASPECT_A).unwrap();

    let plan = graph
        .build_evaluation_plan(&[dependent], EvaluationRequestMode::Default)
        .unwrap();
    let report = graph
        .execute_prepared_plan(&plan, &(), &|ctx| {
            let result = if ctx.node() == source {
                version_ab(1, 0)
            } else {
                version_ab(10, 0)
            };
            Ok::<EvaluationOutput, SignalError>(EvaluationOutput::from_result(result))
        })
        .unwrap();

    assert_eq!(report.task_count, 2);
    assert_eq!(report.tasks_validated_clean, 1, "{report:?}");
    assert_eq!(report.tasks_pruned, 1, "{report:?}");
    assert!(report
        .stages
        .iter()
        .flat_map(|stage| &stage.task_records)
        .any(|record| {
            record.node == dependent
                && matches!(record.outcome, TaskExecutionOutcome::ValidatedClean)
        }));
}

#[test]
fn irrelevant_aspect_change_resolves_before_planning_without_running_compute() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let dependent = graph.node().build();
    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();

    let mut source_v1 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    let mut source_v2_same_aspect_a = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 1));
    let mut dependent_v1 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(10, 0));
    evaluate(&mut graph, source, &mut source_v1).unwrap();
    evaluate(&mut graph, dependent, &mut dependent_v1).unwrap();

    mark_dirty(&mut graph, source, ASPECT_B).unwrap();
    evaluate(&mut graph, source, &mut source_v2_same_aspect_a).unwrap();
    assert_eq!(graph.get_state(dependent).unwrap(), NodeState::Clean);

    let plan = graph
        .build_evaluation_plan(&[dependent], EvaluationRequestMode::Default)
        .unwrap();
    assert_eq!(plan.summary.task_count, 0);

    let calls = AtomicU32::new(0);
    let report = graph
        .execute_prepared_plan(&plan, &(), &|_view| {
            calls.fetch_add(1, Ordering::Relaxed);
            Ok::<EvaluationOutput, SignalError>(EvaluationOutput::from_result(version_ab(99, 0)))
        })
        .unwrap();

    assert_eq!(
        calls.load(Ordering::Relaxed),
        0,
        "MaybeStale validation should skip user compute when cached inputs are still meaningful"
    );
    assert_eq!(report.tasks_validated_clean, 0, "{report:?}");
    assert_eq!(graph.get_state(dependent).unwrap(), NodeState::Clean);
}
