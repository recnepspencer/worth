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
                    ExecutionBudget::new(NonZeroUsize::MIN, memory, work),
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
            assert_eq!(memory.level, MemoryLimitLevel::Policy { ancestor: 0 });
            assert_eq!(memory.admitted, 0);
            assert!(memory.requested > 0);
        }
        assert_eq!(contacts.load(Ordering::SeqCst), 0);
        assert_eq!(graph.node_aspect_version(targets[0]).unwrap(), before);
        assert_eq!(stop.publication_progress().completed_tasks(), 0);
    }
}
