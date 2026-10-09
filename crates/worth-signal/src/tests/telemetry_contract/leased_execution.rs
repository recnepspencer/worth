//! Telemetry derived from carried execution and admitted graph epochs.
use crate::facade::*;
use crate::tests::leased_execution::support::{authority, request};
use crate::tests::support::*;

#[test]
fn transaction_usage_matches_actual_authority_reports() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new();
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let telemetry_session = runtime
        .graph_mut()
        .begin_observation_session(SignalObservationRequest::telemetry())
        .unwrap();
    let nodes = (0..16)
        .map(|_| {
            runtime
                .graph_mut()
                .node()
                .with_contract(
                    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
                )
                .build()
        })
        .collect::<Vec<_>>();
    let mut ctx = ();

    let lease = authority().request_lease(request(4, 10_000_000)).unwrap();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    for &node in &nodes {
        tx.mark_dirty(node, ASPECT_A).unwrap();
    }
    let report = tx
        .evaluate_dirty_checked(
            &|view| Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0)))),
            &lease,
        )
        .unwrap();
    tx.commit().unwrap();

    let metrics = runtime.observe().metrics();
    assert!(report.stages.iter().all(|stage| matches!(
        stage.outcome,
        StageExecutionOutcome::CompletedSerial | StageExecutionOutcome::CompletedParallel
    )));
    let parallel_dispatch = report
        .stages
        .iter()
        .any(|stage| stage.outcome == StageExecutionOutcome::CompletedParallel);
    assert_eq!(
        metrics.execution.serial_executor_usage_count,
        u64::from(!parallel_dispatch)
    );
    assert_eq!(
        metrics.execution.parallel_executor_usage_count,
        u64::from(parallel_dispatch)
    );
    assert_eq!(
        metrics.execution.last_execution_report,
        report.execution.last().copied()
    );
    runtime
        .graph_mut()
        .finish_observation_session(&telemetry_session)
        .unwrap();
}

#[test]
fn checked_epoch_packet_and_reduction_counters_match_consumed_groups() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::operational().with_parallel_admission(
        crate::runtime_policy::ParallelAdmissionPolicy {
            throughput_min_parallel_tasks: 1,
            balanced_min_parallel_tasks: 1,
            latency_bounded_min_parallel_tasks: 1,
            full_parallel_min_tasks: 1,
        },
    ));
    let telemetry_session = graph
        .begin_observation_session(SignalObservationRequest::telemetry())
        .unwrap();
    let requested: Vec<_> = (0..4)
        .map(|_| {
            graph
                .node()
                .with_contract(
                    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
                )
                .build()
        })
        .collect();

    let bootstrap = graph
        .build_evaluation_plan(&requested, EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &|ctx| Ok(ctx.finish(version_ab(1, 0))))
        .unwrap();

    for &node in &requested {
        mark_dirty(&mut graph, node, ASPECT_A).unwrap();
    }

    let plan = graph
        .build_evaluation_plan(&requested, EvaluationRequestMode::Default)
        .unwrap();
    let before = graph.observe().metrics().execution;
    let lease = authority().request_lease(request(4, 1_000_000)).unwrap();
    let report = graph
        .execute_prepared_plan_checked(
            &plan,
            &(),
            &|ctx| Ok(ctx.finish(version_ab(2, 0))),
            worth_execution::ExecutionRequest::leased(&lease),
        )
        .unwrap();
    let groups = report
        .stages
        .iter()
        .map(|stage| u64::from(stage.apply_group_count))
        .sum::<u64>();

    let metrics = graph.observe().metrics();
    let execution = metrics.execution;
    assert_eq!(
        execution.group_local_packet_breadth - before.group_local_packet_breadth,
        4
    );
    assert_eq!(
        execution.reduction_packet_breadth - before.reduction_packet_breadth,
        groups
    );
    assert_eq!(
        execution.reduction_group_count - before.reduction_group_count,
        groups
    );
    assert!(
        execution.shared_surface_publication_breadth - before.shared_surface_publication_breadth
            >= 4,
        "reducer publication breadth should at least cover one semantic publication per task"
    );
    graph
        .finish_observation_session(&telemetry_session)
        .unwrap();
}

#[test]
fn serial_staged_apply_telemetry_tracks_executed_batch_width_not_planned_groups() {
    let mut graph = SignalGraph::new();
    let requested: Vec<_> = (0..4).map(|_| graph.node().build()).collect();

    let bootstrap = graph
        .build_evaluation_plan(&requested, EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &|ctx| Ok(ctx.finish(version_ab(1, 0))))
        .unwrap();

    for &node in &requested {
        mark_dirty(&mut graph, node, ASPECT_A).unwrap();
    }

    let plan = graph
        .build_evaluation_plan(&requested, EvaluationRequestMode::Default)
        .unwrap();
    let before = graph.observe().metrics().execution;
    let report = graph
        .execute_prepared_plan(&plan, &(), &|ctx| Ok(ctx.finish(version_ab(2, 0))))
        .unwrap();

    let serial_stages = report
        .stages
        .iter()
        .filter(|stage| matches!(stage.outcome, StageExecutionOutcome::CompletedSerial))
        .collect::<Vec<_>>();
    assert!(!serial_stages.is_empty());
    assert!(serial_stages
        .iter()
        .all(|stage| stage.apply_group_count == 1));

    let executed_serial_width = serial_stages
        .iter()
        .map(|stage| u64::from(stage.serial_apply_task_count))
        .sum::<u64>();
    let max_serial_batch_width = serial_stages
        .iter()
        .map(|stage| u64::from(stage.serial_apply_task_count))
        .max()
        .unwrap_or(0);

    let after = graph.observe().metrics().execution;
    assert_eq!(
        after.apply_group_width_total - before.apply_group_width_total,
        executed_serial_width
    );
    assert_eq!(
        after.apply_group_disjoint_count - before.apply_group_disjoint_count,
        0
    );
    assert!(
        after.max_apply_group_width >= max_serial_batch_width,
        "global max apply-group width must cover the executed serial batch width"
    );
}
