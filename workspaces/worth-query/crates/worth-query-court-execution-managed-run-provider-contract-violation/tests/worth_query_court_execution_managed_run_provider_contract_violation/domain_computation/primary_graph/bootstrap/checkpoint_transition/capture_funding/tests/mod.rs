use worth_execution::ExecutionAllocationPolicy;
use worth_execution::ExecutionRequest;
use worth_execution::LeaseDenial;
use worth_execution::WorkCeilingDenial;
use worth_query_execution::facade::application_installation::WorthQueryCheckpointTransitionResources;

#[test]
fn serial_backing_without_a_lease_refuses_capture_narrowing() {
    let serial = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    let resources = WorthQueryCheckpointTransitionResources::bounded(1, 1, 1)
        .unwrap()
        .with_capture_memory_limit(0);
    assert!(matches!(
        resources.capture_lease(
            ExecutionRequest::serial(&serial),
            ExecutionAllocationPolicy::SystemAllocation,
        ),
        Err(WorkCeilingDenial::Admission(
            LeaseDenial::NoActiveExecutionScope
        ))
    ));
}
