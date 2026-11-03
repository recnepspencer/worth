use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[test]
fn request_deadline_observation_preserves_the_earliest_active_ancestor_without_mutating_requests() {
    let time = worth_execution::MonotonicTimeController::new().unwrap();
    let held = time.hold();
    let early = time
        .clock()
        .deadline_at(held + std::time::Duration::from_secs(3600));
    let later = time
        .clock()
        .deadline_at(held + std::time::Duration::from_secs(7200));
    let parent = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        Some(early.clone()),
    );
    let child = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        Some(later.clone()),
    );
    let parent_request = ExecutionRequest::serial(&parent);
    let child_request = ExecutionRequest::serial(&child);
    assert_eq!(child_request.deadline(), Some(later.clone()));
    parent_request
        .in_scope(|_| {
            assert_eq!(child_request.deadline(), Some(early.clone()));
            child_request
                .in_scope(|_| assert_eq!(child_request.deadline(), Some(early.clone())))
                .unwrap();
        })
        .unwrap();
    assert_eq!(parent_request.deadline(), Some(early.clone()));
    assert_eq!(child_request.deadline(), Some(later.clone()));
}
