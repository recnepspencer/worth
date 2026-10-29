// The checked door refuses before any callback or graph publication.
pub(crate) use super::{authority, graph};
pub(crate) use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};
pub(crate) use worth_execution::{
    CancellationToken, ExecutionRequest, LeaseRequest, MemoryLimitLevel,
};
pub(crate) use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
pub(crate) use worth_signal::facade::{
    AspectVersion, RunMode, SignalError, SignalExecutionStopReason, SignalLeaseDenial,
};

#[test]
fn declared_result_refusal_never_runs_the_callback() {
    use worth_signal::facade::{
        adapters::NodeContract, Aspect, BoundedSignalInputs, NodeEvaluationResult, SignalGraph,
    };

    for bound in [64, 1 << 20] {
        let mut graph = SignalGraph::new();
        let node = graph
            .node()
            .with_contract(
                NodeContract::wildcard()
                    .with_bounded_inputs(BoundedSignalInputs::default())
                    .with_max_checked_result_heap_bytes(bound),
            )
            .build();
        let memory = worth_execution::SerialMemoryBudget::new(Some(8 << 20));
        let serial = worth_execution::SerialRequest::from_memory(
            memory.clone(),
            CancellationToken::new(),
            None,
        );
        let calls = AtomicUsize::new(0);
        let outcome = graph.evaluate_checked(
            &[node],
            RunMode::Default,
            &(),
            &|context| {
                assert_eq!(context.result_capacity_bytes(), Some(bound));
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(NodeEvaluationResult::from_version(
                    AspectVersion::zero().with(Aspect::new(0), 7),
                )
                .with_output_identity("declared-result"))
            },
            ExecutionRequest::serial(&serial),
        );
        if bound == 64 {
            assert_eq!(outcome.unwrap().tasks_executed, 1);
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            assert_eq!(
                graph.node_aspect_version(node).unwrap().get(Aspect::new(0)),
                7
            );
        } else {
            let SignalError::ExecutionStopped(stop) = outcome.unwrap_err() else {
                panic!("declared result admission retains its typed memory stop");
            };
            assert!(matches!(
                stop.reason(),
                SignalExecutionStopReason::PreparationMemoryExhausted { .. }
                    | SignalExecutionStopReason::Admission(SignalLeaseDenial::MemoryExhausted(_))
            ));
            assert_eq!(calls.load(Ordering::SeqCst), 0);
            assert_eq!(
                graph.node_aspect_version(node).unwrap().get(Aspect::new(0)),
                0
            );
            assert_eq!(stop.publication_progress().completed_tasks(), 0);
        }
        assert_eq!(memory.charged_memory_bytes(), 0);
    }
}
