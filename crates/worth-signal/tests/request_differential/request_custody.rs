//! The checked door refuses before any callback or graph publication.
use super::{authority, graph};
use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};
use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest, MemoryLimitLevel};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_signal::facade::{
    AspectVersion, RunMode, SignalError, SignalExecutionStopReason, SignalLeaseDenial,
};

#[test]
fn zero_budgets_refuse_signal_evaluation_before_callback() {
    for (memory, work) in [(1 << 20, 0), (0, 1_000_000)] {
        let (mut graph, targets) = graph(1);
        let plan = graph
            .build_evaluation_plan(&targets, RunMode::Default)
            .unwrap();
        let before = graph.node_aspect_version(targets[0]).unwrap();
        let contacts = AtomicUsize::new(0);
        let lease = authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Serial,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::MIN, Some(memory), Some(work)),
                ),
                cancellation: CancellationToken::new(),
                deadline: None,
            })
            .unwrap();
        let error = graph
            .execute_prepared_plan_checked(
                &plan,
                &(),
                &|_| {
                    contacts.fetch_add(1, Ordering::SeqCst);
                    Ok(AspectVersion::zero())
                },
                ExecutionRequest::leased(&lease),
            )
            .unwrap_err();
        let SignalError::ExecutionStopped(stop) = error else {
            panic!("the checked door retains its exact stop: {error:?}");
        };
        if work == 0 {
            assert!(
                matches!(
                    stop.reason(),
                    SignalExecutionStopReason::WorkExhausted { .. }
                ),
                "{:?}",
                stop.reason()
            );
        } else {
            let SignalExecutionStopReason::Admission(SignalLeaseDenial::MemoryExhausted(memory)) =
                stop.reason()
            else {
                panic!("native memory admission: {:?}", stop.reason());
            };
            assert!(
                matches!(memory, worth_execution::MemoryLimitDenial::Capacity { admitted: 0, requested, level: MemoryLimitLevel::Policy { ancestor: 0 } } if *requested > 0)
            );
        }
        assert_eq!(contacts.load(Ordering::SeqCst), 0);
        assert_eq!(graph.node_aspect_version(targets[0]).unwrap(), before);
        assert_eq!(stop.publication_progress().completed_tasks(), 0);
    }
}

#[test]
fn singleton_memory_refusal_never_runs_the_callback() {
    // Sweep physical admission boundaries, including diagnostic publication.
    // Each attempt owns its graph and memory, so an earlier success cannot
    // turn a later attempt into a cache hit.
    let mut refused = false;
    let mut completed = false;
    for bytes in [16 << 10, 64 << 10, 256 << 10, 1 << 20, 8 << 20, 32 << 20] {
        let (mut graph, targets) = graph(1);
        let memory = worth_execution::SerialMemoryBudget::new(Some(bytes));
        let serial = worth_execution::SerialRequest::from_memory(
            memory.clone(),
            CancellationToken::new(),
            None,
        );
        let calls = AtomicUsize::new(0);
        let before = graph.node_aspect_version(targets[0]).unwrap();
        let outcome = graph.evaluate_checked(
            &targets,
            RunMode::Default,
            &(),
            &|_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(AspectVersion::zero().with(worth_signal::facade::Aspect::new(0), 7))
            },
            ExecutionRequest::serial(&serial),
        );
        match outcome {
            Ok(report) => {
                completed = true;
                assert_eq!(report.tasks_executed, 1);
                assert_eq!(calls.load(Ordering::SeqCst), 1);
                assert_eq!(
                    graph
                        .node_aspect_version(targets[0])
                        .unwrap()
                        .get(worth_signal::facade::Aspect::new(0)),
                    7
                );
            }
            Err(error) => {
                refused = true;
                assert_eq!(
                    calls.load(Ordering::SeqCst),
                    0,
                    "{bytes}-byte admission refused after callback: {error:?}"
                );
                assert_eq!(graph.node_aspect_version(targets[0]).unwrap(), before);
            }
        }
        assert_eq!(memory.charged_memory_bytes(), 0);
    }
    assert!(refused && completed, "the sweep must cross real admission");
}
