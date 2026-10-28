pub(crate) use std::{
    num::NonZeroUsize,
    sync::{Condvar, Mutex, OnceLock},
    time::Duration,
};
pub(crate) use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionRequest, LeaseRequest, SerialMemoryBudget,
    SerialRequest,
};
pub(crate) use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
pub(crate) use worth_signal::facade::adapters::NodeContract;
pub(crate) use worth_signal::facade::specialist::ExecutionReport;
pub(crate) use worth_signal::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, RunMode as EvaluationRequestMode, SignalError,
    SignalGraph,
};

pub(crate) fn authority() -> &'static ExecutionAuthority {
    static OWNER: OnceLock<ExecutionAuthority> = OnceLock::new();
    OWNER.get_or_init(|| {
        ExecutionAuthority::try_construct(worth_execution::ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(128 << 20),
        })
        .unwrap()
    })
}

pub(crate) fn graph(count: usize) -> (SignalGraph, Vec<worth_signal::facade::NodeId>) {
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

pub(crate) fn serial(count: usize) -> Result<(Vec<AspectVersion>, ExecutionReport), SignalError> {
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

pub(crate) fn checked_request(
    count: usize,
    workers: usize,
    work: u64,
) -> Result<(Vec<AspectVersion>, ExecutionReport), SignalError> {
    let (mut graph, targets) = graph(count);
    let plan = graph.build_evaluation_plan(&targets, EvaluationRequestMode::Default)?;
    let lease = (workers > 0).then(|| {
        authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    if workers == 1 {
                        ExecutionPosture::Serial
                    } else {
                        ExecutionPosture::Automatic
                    },
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::new(workers).unwrap(), None, Some(work)),
                ),
                cancellation: CancellationToken::new(),
                deadline: None,
            })
            .unwrap()
    });
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let request = lease.as_ref().map_or_else(
        || ExecutionRequest::serial(&serial),
        ExecutionRequest::leased,
    );
    let participants = Mutex::new(Vec::new());
    let rendezvous = (Mutex::new(0_usize), Condvar::new());
    let mut evaluate = || {
        graph.execute_prepared_plan_checked(
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
            request,
        )
    };
    let report = if workers == 0 {
        worth_execution::ExecutionWorkCeiling::new(Some(work))
            .run_serial(&serial, evaluate)
            .map_err(SignalError::execution_scope_denied)?
            .0?
    } else {
        evaluate()?
    };
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
#[path = "request_differential/request_custody.rs"]
pub(crate) mod request_custody;
