use super::support::{authority, request};
use crate::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, DeclaredSignalInput, EvaluationRequestMode,
    NodeContract, SignalGraph,
};

const VALUE: Aspect = Aspect::new(0);

#[test]
fn stopped_prepared_plan_replaces_the_previous_successful_execution_report() {
    let mut graph = SignalGraph::new();
    let observation = graph
        .begin_observation_session(crate::facade::SignalObservationRequest::telemetry())
        .unwrap();
    let node = graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    let lease = authority().request_lease(request(1, 1_000_000)).unwrap();
    let success = graph
        .evaluate_checked(
            &[node],
            EvaluationRequestMode::Default,
            &(),
            &|_| Ok(AspectVersion::zero().with(VALUE, 1)),
            &lease,
        )
        .unwrap();
    assert_eq!(
        graph.observe().telemetry().execution.last_execution_report,
        success.execution.last().copied(),
    );
    let target = graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    let plan = graph
        .build_evaluation_plan(&[target], EvaluationRequestMode::Default)
        .unwrap();
    let exhausted = authority().request_lease(request(1, 0)).unwrap();
    let error = graph
        .execute_prepared_plan_checked(
            &plan,
            &(),
            &|_| -> Result<AspectVersion, crate::facade::SignalError> {
                panic!("an exhausted request must stop before evaluation");
            },
            &exhausted,
        )
        .unwrap_err();
    let crate::facade::SignalError::ExecutionStopped(stop) = error else {
        panic!("exhaustion must preserve its enclosing execution report");
    };
    assert_ne!(Some(stop.execution()), success.execution.last().copied());
    assert_eq!(
        graph.observe().telemetry().execution.last_execution_report,
        Some(stop.execution()),
    );
    assert_eq!(stop.publication_progress().completed_tasks(), 0);
    assert_eq!(graph.node_aspect_version(target).unwrap().get(VALUE), 0);
    graph.finish_observation_session(&observation).unwrap();
}

#[test]
fn empty_request_returns_its_enclosing_authority_report_without_evaluation() {
    let mut graph = SignalGraph::new();
    let lease = authority().request_lease(request(1, 1_000_000)).unwrap();
    let report = graph
        .evaluate_checked(
            &[],
            EvaluationRequestMode::Default,
            &(),
            &|_| -> Result<AspectVersion, crate::facade::SignalError> {
                panic!("empty request has no evaluator work");
            },
            &lease,
        )
        .unwrap();
    assert_eq!(report.tasks_executed, 0);
    assert_eq!(report.execution.len(), 1);
    assert!(
        report.execution[0]
            .physical()
            .active_workers_high_watermark()
            <= 1
    );
}

#[test]
fn two_epochs_retain_a_slot_for_the_enclosing_physical_report() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(
        crate::facade::SignalRuntimePolicy::forensic().with_parallel_admission(
            crate::facade::ParallelAdmissionPolicy {
                throughput_min_parallel_tasks: 1,
                balanced_min_parallel_tasks: 1,
                latency_bounded_min_parallel_tasks: 1,
                full_parallel_min_tasks: 1,
            },
        ),
    );
    let source = graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    let target = graph
        .node()
        .with_contract(
            NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new([
                DeclaredSignalInput::new(source, VALUE),
            ])),
        )
        .build();
    let lease = authority().request_lease(request(4, 1_000_000)).unwrap();
    let report = graph
        .evaluate_checked(
            &[target],
            EvaluationRequestMode::Default,
            &(),
            &|ctx| {
                let value = if ctx.node() == source {
                    5
                } else {
                    ctx.read(source, VALUE)? + 1
                };
                Ok(AspectVersion::zero().with(VALUE, value))
            },
            &lease,
        )
        .unwrap();
    assert_eq!(report.stages.len(), 2);
    assert_eq!(report.tasks_executed, 2);
    assert_eq!(graph.node_aspect_version(target).unwrap().get(VALUE), 6);
    assert!(report.execution.len() >= 3);
    assert!(report.execution[..report.execution.len() - 1]
        .iter()
        .all(|child| child.resolved_posture() == worth_foundational::ExecutionPosture::Serial));
    assert!(report.stages.iter().all(
        |stage| stage.outcome == crate::logic::planner::StageExecutionOutcome::CompletedSerial
    ));
    let enclosing_workers = report
        .execution
        .last()
        .unwrap()
        .physical()
        .active_workers_high_watermark();
    assert!(enclosing_workers > 0 && enclosing_workers <= 4);
}
