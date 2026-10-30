pub(crate) use worth_signal::facade::adapters::NodeContract;
pub(crate) use worth_signal::facade::specialist::ExecutionReport;
pub(crate) use {
    std::num::NonZeroUsize, std::sync::Condvar, std::sync::Mutex, std::sync::OnceLock,
    std::time::Duration,
};
pub(crate) use {
    worth_execution::CancellationToken, worth_execution::ExecutionAuthority,
    worth_execution::ExecutionRequest, worth_execution::LeaseRequest,
    worth_execution::SerialMemoryBudget, worth_execution::SerialRequest,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
};
pub(crate) use {
    worth_signal::facade::Aspect, worth_signal::facade::AspectVersion,
    worth_signal::facade::BoundedSignalInputs,
    worth_signal::facade::RunMode as EvaluationRequestMode, worth_signal::facade::SignalError,
    worth_signal::facade::SignalGraph,
};

pub(crate) fn authority() -> &'static ExecutionAuthority {
    static OWNER: OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
    OWNER.get_or_init(|| {
        worth_execution::ExecutionAuthority::try_construct(
            worth_execution::ExecutionAuthorityConfig {
                max_workers: std::num::NonZeroUsize::new(4).unwrap(),
                charged_memory_bytes: Some(128 << 20),
            },
        )
        .unwrap()
    })
}

pub(crate) fn graph(count: usize) -> (SignalGraph, Vec<worth_signal::facade::NodeId>) {
    let mut graph = worth_signal::facade::SignalGraph::new();
    let targets = (0..count)
        .map(|_| {
            graph
                .node()
                .with_contract(
                    worth_signal::facade::adapters::NodeContract::wildcard()
                        .with_bounded_inputs(worth_signal::facade::BoundedSignalInputs::default()),
                )
                .build()
        })
        .collect();
    (graph, targets)
}

pub(crate) fn serial(count: usize) -> Result<(Vec<AspectVersion>, ExecutionReport), SignalError> {
    let (mut graph, targets) = graph(count);
    let plan = graph.build_evaluation_plan(&targets, worth_signal::facade::RunMode::Default)?;
    let report = graph.execute_prepared_plan(&plan, &(), &|context| {
        let index = targets
            .iter()
            .position(|node| *node == context.node())
            .unwrap();
        Ok(worth_signal::facade::AspectVersion::zero()
            .with(worth_signal::facade::Aspect::new(0), index as u64 + 1))
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
    let plan = graph.build_evaluation_plan(&targets, worth_signal::facade::RunMode::Default)?;
    let lease = (workers > 0).then(|| {
        authority()
            .request_lease(LeaseRequest {
                policy: worth_foundational::ExecutionRequestPolicy::new(
                    if workers == 1 {
                        worth_foundational::ExecutionPosture::Serial
                    } else {
                        worth_foundational::ExecutionPosture::Automatic
                    },
                    worth_foundational::DeterminismContract::CanonicalBitwise,
                    worth_foundational::ExecutionBudget::new(
                        std::num::NonZeroUsize::new(workers).unwrap(),
                        None,
                        Some(work),
                    ),
                ),
                cancellation: worth_execution::CancellationToken::new(),
                deadline: None,
            })
            .unwrap()
    });
    let serial = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request = lease.as_ref().map_or_else(
        || worth_execution::ExecutionRequest::serial(&serial),
        worth_execution::ExecutionRequest::leased,
    );
    let participants = std::sync::Mutex::new(Vec::new());
    let rendezvous = (std::sync::Mutex::new(0_usize), std::sync::Condvar::new());
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
                            .wait_timeout_while(ready, std::time::Duration::from_secs(5), |n| {
                                *n < 2
                            })
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
                Ok(worth_signal::facade::AspectVersion::zero()
                    .with(worth_signal::facade::Aspect::new(0), index as u64 + 1))
            },
            request,
        )
    };
    let report = if workers == 0 {
        worth_execution::ExecutionWorkCeiling::new(Some(work))
            .run_serial(&serial, evaluate)
            .map_err(worth_signal::facade::SignalError::execution_scope_denied)?
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
