use crate::facade::{
    mark_dirty, EvaluationRequestMode, ParallelAdmissionPolicy, SignalGraph, SignalRuntimePolicy,
    StageExecutionOutcome,
};
use crate::tests::leased_execution::support::{authority, request};
use crate::tests::support::{version_ab, ASPECT_A};

use super::executor_policy::{aggressive_parallel_runtime_policy, bounded_contract};

#[test]
fn many_thin_stages_use_one_worker_with_bounded_physical_memory_growth() {
    let mut peaks = Vec::new();
    for length in [8, 32] {
        let mut graph = SignalGraph::new();
        let mut chain = Vec::new();
        for _ in 0..length {
            let inputs = chain.last().copied().into_iter().collect::<Vec<_>>();
            chain.push(
                graph
                    .node()
                    .with_contract(bounded_contract(&inputs))
                    .build(),
            );
        }
        let lease = authority().request_lease(request(4, 2_000_000)).unwrap();
        let report = graph
            .evaluate_checked(
                &[chain[length - 1]],
                EvaluationRequestMode::Default,
                &(),
                &|ctx| {
                    let position = chain.iter().position(|&node| node == ctx.node()).unwrap();
                    let value = if position == 0 {
                        1
                    } else {
                        ctx.read(chain[position - 1], ASPECT_A)? + 1
                    };
                    Ok(version_ab(value, 0))
                },
                worth_execution::ExecutionRequest::leased(&lease),
            )
            .unwrap();
        assert_eq!(
            graph
                .node_aspect_version(chain[length - 1])
                .unwrap()
                .get(ASPECT_A),
            length as u64
        );
        assert_eq!(report.tasks_executed, length as u32);
        assert!(report
            .stages
            .iter()
            .all(|stage| stage.outcome == StageExecutionOutcome::CompletedSerial));
        assert!(report
            .execution
            .iter()
            .all(|execution| execution.physical().active_workers_high_watermark() <= 1));
        peaks.push(
            report
                .execution
                .iter()
                .map(|execution| execution.physical().peak_charged_memory_bytes())
                .max()
                .unwrap(),
        );
    }
    // Growing from eight to thirty-two stages keeps observed physical memory
    // growth within one MiB, including plan and report storage.
    assert!(
        peaks[1] <= peaks[0] + 1024 * 1024,
        "physical memory growth exceeded the bound: {peaks:?}"
    );
}

#[test]
fn wide_checked_epoch_reports_resolved_parallel_admission_and_bounded_workers() {
    if crate::tests::leased_execution::support::private_authority_process::run_with_private_authority(
        "tests::adversarial_parallel::parallel_admission::wide_checked_epoch_reports_resolved_parallel_admission_and_bounded_workers",
    ) {
        return;
    }
    let rendezvous =
        crate::tests::leased_execution::support::task_rendezvous::TaskRendezvous::default();
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(aggressive_parallel_runtime_policy());
    let nodes = (0..16)
        .map(|_| graph.node().with_contract(bounded_contract(&[])).build())
        .collect::<Vec<_>>();
    let lease = authority().request_lease(request(4, 2_000_000)).unwrap();
    let report = graph
        .evaluate_checked(
            &nodes,
            EvaluationRequestMode::Default,
            &(),
            &|ctx| {
                rendezvous.meet();
                for _ in 0..4096 {
                    ctx.work().checkpoint(1).map_err(|_| {
                        crate::facade::SignalError::invalid_input("test kernel stopped")
                    })?;
                    std::thread::yield_now();
                }
                Ok(version_ab(ctx.node().index() as u64 + 1, 0))
            },
            worth_execution::ExecutionRequest::leased(&lease),
        )
        .unwrap();
    let workers = report
        .execution
        .iter()
        .map(|execution| execution.physical().active_workers_high_watermark())
        .max()
        .unwrap();
    let resolved_parallel = report.execution.iter().any(|execution| {
        execution.resolved_posture() == worth_foundational::ExecutionPosture::Automatic
    });
    // A private process authority has four workers; sixteen tasks exceed the installed one-task thresholds.
    assert!(resolved_parallel);
    rendezvous.assert_if_parallel(resolved_parallel);
    assert!(workers >= 1 && workers <= lease.policy().budget().max_workers().get());
    assert_eq!(
        report
            .stages
            .iter()
            .any(|stage| stage.outcome == StageExecutionOutcome::CompletedParallel),
        resolved_parallel
    );
    if resolved_parallel {
        assert!(graph.telemetry().execution.parallel_stage_dispatch_count > 0);
    }
}

#[test]
fn one_worker_lease_does_not_report_parallel_dispatch_for_wide_work() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(aggressive_parallel_runtime_policy());
    let nodes = (0..16)
        .map(|_| graph.node().with_contract(bounded_contract(&[])).build())
        .collect::<Vec<_>>();
    let lease = authority().request_lease(request(1, 2_000_000)).unwrap();
    let report = graph
        .evaluate_checked(
            &nodes,
            EvaluationRequestMode::Default,
            &(),
            &|ctx| Ok(version_ab(ctx.node().index() as u64 + 1, 0)),
            worth_execution::ExecutionRequest::leased(&lease),
        )
        .unwrap();
    assert!(report
        .execution
        .iter()
        .all(|execution| execution.physical().active_workers_high_watermark() <= 1));
    assert!(report
        .stages
        .iter()
        .all(|stage| stage.outcome == StageExecutionOutcome::CompletedSerial));
    assert_eq!(graph.telemetry().execution.parallel_stage_dispatch_count, 0);
}

#[test]
fn installed_threshold_keeps_small_checked_epochs_serial() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::operational().with_parallel_admission(
        ParallelAdmissionPolicy {
            throughput_min_parallel_tasks: 8,
            balanced_min_parallel_tasks: 8,
            latency_bounded_min_parallel_tasks: 8,
            full_parallel_min_tasks: 8,
        },
    ));
    let nodes = std::array::from_fn::<_, 2, _>(|_| {
        graph.node().with_contract(bounded_contract(&[])).build()
    });
    let plan = graph
        .build_evaluation_plan(&nodes, EvaluationRequestMode::Default)
        .unwrap();
    let lease = authority().request_lease(request(4, 2_000_000)).unwrap();
    let report = graph
        .execute_prepared_plan_checked(
            &plan,
            &(),
            &|_| Ok(version_ab(2, 0)),
            worth_execution::ExecutionRequest::leased(&lease),
        )
        .unwrap();
    assert!(report
        .execution
        .iter()
        .all(|execution| execution.physical().active_workers_high_watermark() <= 1));
    assert!(report
        .stages
        .iter()
        .all(|stage| stage.outcome == StageExecutionOutcome::CompletedSerial));
    assert_eq!(graph.telemetry().execution.parallel_stage_dispatch_count, 0);
    for node in nodes {
        mark_dirty(&mut graph, node, ASPECT_A).unwrap();
        assert_eq!(graph.node_aspect_version(node).unwrap().get(ASPECT_A), 2);
    }
}
