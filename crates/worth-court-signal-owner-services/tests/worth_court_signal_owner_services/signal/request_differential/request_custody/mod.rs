// The checked door refuses before any callback or graph publication.
pub(crate) use {
    crate::request_differential::ordinary_context::authority,
    crate::request_differential::ordinary_context::graph,
};
pub(crate) use {
    std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering,
};
pub(crate) use {
    worth_execution::CancellationToken, worth_execution::ExecutionRequest,
    worth_execution::LeaseRequest, worth_execution::MemoryLimitLevel,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
};
pub(crate) use {
    worth_signal::facade::AspectVersion, worth_signal::facade::RunMode,
    worth_signal::facade::SignalError, worth_signal::facade::SignalExecutionStopReason,
    worth_signal::facade::SignalLeaseDenial,
};

#[test]
fn declared_result_refusal_never_runs_the_callback() {
    use {
        worth_signal::facade::adapters::NodeContract, worth_signal::facade::Aspect,
        worth_signal::facade::BoundedSignalInputs, worth_signal::facade::NodeEvaluationResult,
        worth_signal::facade::SignalGraph,
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
            worth_execution::CancellationToken::new(),
            None,
        );
        let calls = std::sync::atomic::AtomicUsize::new(0);
        let outcome = graph.evaluate_checked(
            &[node],
            worth_signal::facade::RunMode::Default,
            &(),
            &|context| {
                assert_eq!(context.result_capacity_bytes(), Some(bound));
                calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(NodeEvaluationResult::from_version(
                    worth_signal::facade::AspectVersion::zero().with(Aspect::new(0), 7),
                )
                .with_output_identity("declared-result"))
            },
            worth_execution::ExecutionRequest::serial(&serial),
        );
        if bound == 64 {
            assert_eq!(outcome.unwrap().tasks_executed, 1);
            assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
            assert_eq!(
                graph.node_aspect_version(node).unwrap().get(Aspect::new(0)),
                7
            );
        } else {
            let worth_signal::facade::SignalError::ExecutionStopped(stop) = outcome.unwrap_err()
            else {
                panic!("declared result admission retains its typed memory stop");
            };
            assert!(matches!(
                stop.reason(),
                worth_signal::facade::SignalExecutionStopReason::PreparationMemoryExhausted { .. }
                    | worth_signal::facade::SignalExecutionStopReason::Admission(
                        worth_signal::facade::SignalLeaseDenial::MemoryExhausted(_)
                    )
            ));
            assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
            assert_eq!(
                graph.node_aspect_version(node).unwrap().get(Aspect::new(0)),
                0
            );
            assert_eq!(stop.publication_progress().completed_tasks(), 0);
        }
        assert_eq!(memory.charged_memory_bytes(), 0);
    }
}
