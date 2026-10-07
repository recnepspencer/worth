use super::planner_temporal_resolver::PlannerTemporalResolver;
use crate::facade::{
    build_evaluation_plan_with_policy_resolver, execute_plan_with_policy_and_condition, Aspect,
    AspectVersion, ClockAdvanceRequest, ClockDomain, ClockTick, DefaultComparatorPolicyResolver,
    EvaluationOutput, EvaluationRequestMode, NodeEvaluationResult, NodeId, NodeState, SignalError,
    SignalGraph, SignalRuntime, TaskExecutionOutcome, TemporalEligibilityAuthority,
};
use crate::tests::support::version_ab;

#[test]
fn execution_report_marks_on_demand_deferral_explicitly() {
    let mut graph = SignalGraph::new();
    let node = graph.node().on_demand().build();

    let plan = graph
        .build_evaluation_plan(&[node], EvaluationRequestMode::Default)
        .unwrap();
    let report = graph
        .execute_prepared_plan(&plan, &(), &|_ctx| {
            Ok::<EvaluationOutput, SignalError>(EvaluationOutput::deferred_by_condition())
        })
        .unwrap();

    assert_eq!(report.task_count, 1);
    assert_eq!(report.tasks_deferred_by_condition, 1);
    assert!(matches!(
        report.stages[0].task_records[0].outcome,
        TaskExecutionOutcome::ConditionDeferred
    ));
    assert_eq!(graph.get_state(node).unwrap(), NodeState::MaybeStale);
}

#[test]
fn execution_report_marks_temporal_deferral_explicitly() {
    let mut graph = SignalGraph::new();
    let node = graph.node().debounce(50).unwrap().build();
    let mut comparator = DefaultComparatorPolicyResolver::default();
    let plan = build_evaluation_plan_with_policy_resolver(
        &mut graph,
        &[node],
        EvaluationRequestMode::Default,
        &mut comparator,
    )
    .unwrap();

    let mut resolver = PlannerTemporalResolver::default();
    let report = execute_plan_with_policy_and_condition(
        &mut graph,
        &plan,
        &mut |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(9, 0)),
        &mut comparator,
        &mut resolver,
        None,
    )
    .unwrap();

    assert_eq!(report.tasks_deferred_by_condition, 1);
    assert!(matches!(
        report.stages[0].task_records[0].deferral_reason,
        Some(crate::logic::evaluation::DeferralReason::TemporalConditionNotMet)
    ));
    let temporal = report.stages[0].task_records[0]
        .temporal_eligibility
        .as_ref()
        .expect("temporal deferral should preserve lowered temporal proof");
    assert_eq!(
        temporal.authority(),
        TemporalEligibilityAuthority::RuntimeClockBasis
    );
    assert_eq!(report.temporal_summary.total_count(), 1);
    assert_eq!(report.temporal_summary.deferred_count(), 1);
    assert_eq!(report.temporal_summary.resolver_fallback_count(), 0);
    assert_eq!(
        graph
            .observe()
            .metrics()
            .temporal
            .temporal_eligibility_lowering_count,
        1
    );
}

#[test]
fn runtime_clock_backed_temporal_ready_is_reported_as_runtime_authority() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let node = runtime.graph_mut().node().at_or_after(5).build();
    let plan = runtime
        .build_evaluation_plan(&[node], EvaluationRequestMode::Default)
        .unwrap();
    let aspect = Aspect::new(0);

    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(5),
        ))
        .unwrap();

    let report = runtime
        .execute_prepared_plan(&plan, &(), &|_ctx| {
            Ok::<EvaluationOutput, SignalError>(EvaluationOutput::from_result(
                NodeEvaluationResult::from_version(AspectVersion::from_updates([(aspect, 1)])),
            ))
        })
        .unwrap();

    let temporal = report.stages[0].task_records[0]
        .temporal_eligibility
        .as_ref()
        .expect("runtime-backed temporal admission should survive into task record");
    assert_eq!(
        temporal.authority(),
        TemporalEligibilityAuthority::RuntimeClockBasis
    );
    assert_eq!(temporal.authority_tick(), Some(ClockTick::new(5)));
    assert!(temporal.ready_by_time());
    assert_eq!(report.temporal_summary.total_count(), 1);
    assert_eq!(report.temporal_summary.ready_count(), 1);
    assert_eq!(report.temporal_summary.runtime_clock_authority_count(), 1);
}
