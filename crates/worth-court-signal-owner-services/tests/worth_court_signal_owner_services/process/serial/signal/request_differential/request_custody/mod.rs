// The checked door refuses before any callback or graph publication.
#[allow(unused_imports)]
pub(crate) use {worth_signal::facade::AspectVersion, std::sync::atomic::AtomicUsize, worth_execution::CancellationToken, worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_execution::ExecutionRequest, worth_foundational::ExecutionRequestPolicy, worth_execution::LeaseRequest, worth_execution::MemoryLimitLevel, std::num::NonZeroUsize, std::sync::atomic::Ordering, worth_signal::facade::RunMode, worth_signal::facade::SignalError, worth_signal::facade::SignalExecutionStopReason, worth_signal::facade::SignalLeaseDenial, crate::request_differential::ordinary_context::authority, crate::request_differential::ordinary_context::graph};

#[test]
fn zero_budgets_refuse_signal_evaluation_before_callback() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    for (memory, work) in [(1 << 20, 0), (0, 1_000_000)] {
        let (mut graph, targets) = graph(1);
        let plan = graph
            .build_evaluation_plan(&targets, worth_signal::facade::RunMode::Default)
            .unwrap();
        let before = graph.node_aspect_version(targets[0]).unwrap();
        let contacts = std::sync::atomic::AtomicUsize::new(0);
        let lease = authority()
            .request_lease(LeaseRequest {
                policy: worth_foundational::ExecutionRequestPolicy::new(
                    worth_foundational::ExecutionPosture::Serial,
                    worth_foundational::DeterminismContract::CanonicalBitwise,
                    worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(memory), Some(work)),
                ),
                cancellation: worth_execution::CancellationToken::new(),
                deadline: None,
            })
            .unwrap();
        let error = graph
            .execute_prepared_plan_checked(
                &plan,
                &(),
                &|_| {
                    contacts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(worth_signal::facade::AspectVersion::zero())
                },
                worth_execution::ExecutionRequest::leased(&lease),
            )
            .unwrap_err();
        let worth_signal::facade::SignalError::ExecutionStopped(stop) = error else {
            panic!("the checked door retains its exact stop: {error:?}");
        };
        if work == 0 {
            assert!(
                matches!(
                    stop.reason(),
                    worth_signal::facade::SignalExecutionStopReason::WorkExhausted { .. }
                ),
                "{:?}",
                stop.reason()
            );
        } else {
            let worth_signal::facade::SignalExecutionStopReason::Admission(worth_signal::facade::SignalLeaseDenial::MemoryExhausted(memory)) =
                stop.reason()
            else {
                panic!("native memory admission: {:?}", stop.reason());
            };
            assert!(
                matches!(memory, worth_execution::MemoryLimitDenial::Capacity { admitted: 0, requested, level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 } } if *requested > 0)
            );
        }
        assert_eq!(contacts.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert_eq!(graph.node_aspect_version(targets[0]).unwrap(), before);
        assert_eq!(stop.publication_progress().completed_tasks(), 0);
    }
}
