use std::{
    num::NonZeroUsize,
    sync::{Condvar, Mutex, OnceLock},
    time::Duration,
};
use worth_execution::{CancellationToken, ExecutionAuthority, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_signal::facade::adapters::NodeContract;
use worth_signal::facade::specialist::ExecutionReport;
use worth_signal::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, RunMode as EvaluationRequestMode, SignalError,
    SignalGraph,
};

fn authority() -> &'static ExecutionAuthority {
    static OWNER: OnceLock<ExecutionAuthority> = OnceLock::new();
    OWNER.get_or_init(|| {
        ExecutionAuthority::try_construct(worth_execution::ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(128 << 20),
        })
        .unwrap()
    })
}

fn graph(count: usize) -> (SignalGraph, Vec<worth_signal::facade::NodeId>) {
    let mut graph = SignalGraph::new();
    let targets = (0..count)
        .map(|_| {
            graph
                .node()
                .with_contract(
                    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
                )
                .build()
        })
        .collect();
    (graph, targets)
}

fn serial(count: usize) -> Result<(Vec<AspectVersion>, ExecutionReport), SignalError> {
    let (mut graph, targets) = graph(count);
    let plan = graph.build_evaluation_plan(&targets, EvaluationRequestMode::Default)?;
    let report = graph.execute_prepared_plan(&plan, &(), &|context| {
        let index = targets
            .iter()
            .position(|node| *node == context.node())
            .unwrap();
        Ok(AspectVersion::zero().with(Aspect::new(0), index as u64 + 1))
    })?;
    assert_eq!(report.tasks_executed, count as u32);
    assert_eq!(
        graph.observe().telemetry().execution.last_execution_report,
        report.execution.last().copied()
    );
    let values = targets
        .iter()
        .map(|node| graph.node_aspect_version(*node).unwrap())
        .collect();
    Ok((values, report))
}

fn leased(
    count: usize,
    workers: usize,
    work: u64,
) -> Result<(Vec<AspectVersion>, ExecutionReport), SignalError> {
    let (mut graph, targets) = graph(count);
    let plan = graph.build_evaluation_plan(&targets, EvaluationRequestMode::Default)?;
    let lease = authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                if workers == 1 {
                    ExecutionPosture::Serial
                } else {
                    ExecutionPosture::Automatic
                },
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(workers).unwrap(), 64 << 20, work),
            ),
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let participants = Mutex::new(Vec::new());
    let rendezvous = (Mutex::new(0_usize), Condvar::new());
    let report = graph.execute_prepared_plan_checked(
        &plan,
        &(),
        &|context| {
            if workers > 1 {
                let thread = std::thread::current().id();
                let join = {
                    let mut seen = participants.lock().unwrap();
                    if seen.contains(&thread) || seen.len() >= 2 {
                        false
                    } else {
                        seen.push(thread);
                        true
                    }
                };
                if join {
                    let (ready, wake) = &rendezvous;
                    let mut ready = ready.lock().unwrap();
                    *ready += 1;
                    wake.notify_all();
                    let (ready, _) = wake
                        .wait_timeout_while(ready, Duration::from_secs(5), |n| *n < 2)
                        .unwrap();
                    assert_eq!(
                        *ready, 2,
                        "two workers must rendezvous before the bounded wait ends"
                    );
                }
            }
            let index = targets
                .iter()
                .position(|node| *node == context.node())
                .unwrap();
            Ok(AspectVersion::zero().with(Aspect::new(0), index as u64 + 1))
        },
        &lease,
    )?;
    if workers > 1 {
        assert_eq!(*rendezvous.0.lock().unwrap(), 2);
    }
    assert_eq!(report.tasks_executed, count as u32);
    let values = targets
        .iter()
        .map(|node| graph.node_aspect_version(*node).unwrap())
        .collect();
    Ok((values, report))
}

#[test]
fn serial_default_admits_three_hundred_tasks_and_reports_their_charge() {
    let (values, report) = serial(300).unwrap();
    for (index, value) in values.iter().enumerate() {
        assert_eq!(value.get(Aspect::new(0)), index as u64 + 1);
    }
    assert!(report.execution.last().unwrap().charged_work() > 0);
}

// The ordinary entry publishes one task per epoch and the checked entry
// publishes grouped epochs, so the two charge different work for one plan.
// Results are the law here. Charged work is compared between requests that
// enter the same door: one worker and many, below.
#[test]
fn serial_entry_and_one_worker_lease_have_identical_results() {
    let serial = serial(300).unwrap();
    let one = leased(300, 1, u64::MAX).unwrap();
    assert_eq!(serial.0, one.0);
    assert!(serial.1.execution.last().unwrap().charged_work() > 0);
    assert!(one.1.execution.last().unwrap().charged_work() > 0);
}

#[test]
fn request_results_and_charged_work_are_identical_at_one_and_many_workers() {
    let one = leased(32, 1, u64::MAX).unwrap();
    let many = leased(32, 4, u64::MAX).unwrap();
    assert_eq!(one.0, many.0);
    assert_eq!(
        one.1.execution.last().unwrap().charged_work(),
        many.1.execution.last().unwrap().charged_work()
    );
}

#[test]
fn leased_three_hundred_tasks_still_refuse_a_callers_work_ceiling() {
    // One unit cannot cover the required per-item checkpoints of 300 tasks.
    // This caller allowance is independent of a measured successful report.
    let SignalError::ExecutionStopped(stop) = leased(300, 1, 1).unwrap_err() else {
        panic!("the caller's finite work ceiling must refuse the evaluation");
    };
    assert!(matches!(
        stop.reason(),
        worth_signal::facade::SignalExecutionStopReason::WorkExhausted { .. }
            | worth_signal::facade::SignalExecutionStopReason::Failure {
                cause: worth_signal::facade::SignalExecutionFailure::WorkCeiling,
                ..
            }
    ));
}
