use crate::facade::{SignalError, SignalLeaseDenial as Own};

#[test]
fn serial_planning_refuses_the_required_policy_memory() {
    let mut graph = crate::facade::SignalGraph::new();
    let mut policy = crate::facade::SignalRuntimePolicy::development();
    policy.serial_memory_bytes = 1;
    graph.set_runtime_policy(policy);
    let error = graph
        .build_evaluation_plan(&[], crate::facade::EvaluationRequestMode::Default)
        .unwrap_err();
    let SignalError::ExecutionAdmissionDenied(Own::MemoryExhausted(memory)) = error else {
        panic!("bounded serial preparation must preserve the memory cause");
    };
    assert_eq!(memory.admitted, 1);
    assert_eq!(
        memory.level,
        worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
    );
}

#[test]
fn serial_callback_returns_its_original_heap_owned_domain_error() {
    use crate::facade::{AspectVersion, EvaluationRequestMode, SignalGraph};
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let plan = graph
        .build_evaluation_plan(&[node], EvaluationRequestMode::Default)
        .unwrap();
    let expected = SignalError::invalid_input(String::from("semantic callback refusal"));
    assert!(worth_execution::ChargedBytes::additional_charged_bytes(&expected) > 0);
    let error = graph
        .execute_prepared_plan(&plan, &(), &|_| {
            Err::<AspectVersion, _>(SignalError::invalid_input(String::from(
                "semantic callback refusal",
            )))
        })
        .unwrap_err();
    assert_eq!(error, expected);
}

#[test]
fn supplied_serial_cancellation_stops_after_the_evaluator_runs() {
    use crate::facade::{AspectVersion, EvaluationRequestMode, SignalGraph};
    let mut graph = SignalGraph::new();
    let targets = (0..300).map(|_| graph.node().build()).collect::<Vec<_>>();
    let plan = graph
        .build_evaluation_plan(&targets, EvaluationRequestMode::Default)
        .unwrap();
    let cancellation = worth_execution::CancellationSource::new();
    let request = graph
        .bounded_serial_request()
        .with_cancellation(cancellation.token());
    let mut hook = crate::data::comparator::DefaultComparatorResolver;
    let mut resolver = crate::data::comparator::DefaultComparatorPolicyResolver {
        fallback: crate::data::comparator::VersionComparatorPolicy::Exact,
        custom: &mut hook,
    };
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let error = crate::logic::planner::execute_prepared_plan_with_policy(
        &mut graph,
        &plan,
        &(),
        &|_| {
            calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            cancellation.cancel();
            Ok(AspectVersion::zero())
        },
        &mut resolver,
        worth_execution::ExecutionRequest::serial(&request),
    )
    .unwrap_err();
    assert_eq!(calls.into_inner(), 1);
    let SignalError::ExecutionStopped(stop) = error else {
        panic!("request must stop")
    };
    assert!(matches!(
        stop.reason(),
        crate::data::error::SignalExecutionStopReason::Failure {
            cause: crate::data::error::SignalExecutionFailure::Cancelled,
            ..
        }
    ));
}

#[test]
fn serial_three_hundred_tasks_still_refuse_insufficient_memory() {
    use crate::facade::{EvaluationRequestMode, SignalGraph};
    let mut graph = SignalGraph::new();
    let targets = (0..300).map(|_| graph.node().build()).collect::<Vec<_>>();
    let mut policy = graph.runtime_policy();
    policy.serial_memory_bytes = 4096;
    graph.set_runtime_policy(policy);
    let error = graph
        .build_evaluation_plan(&targets, EvaluationRequestMode::Default)
        .unwrap_err();
    assert!(
        matches!(
            error,
            SignalError::ExecutionAdmissionDenied(Own::MemoryExhausted(_))
                | SignalError::PreparationMemoryExhausted { .. }
                | SignalError::ExecutionStopped(_)
        ),
        "memory must refuse the evaluation: {error:?}"
    );
    if let SignalError::ExecutionStopped(stop) = error {
        assert!(matches!(
            stop.reason(),
            crate::data::error::SignalExecutionStopReason::PreparationMemoryExhausted { .. }
                | crate::data::error::SignalExecutionStopReason::Admission(Own::MemoryExhausted(_))
        ));
    }
}
