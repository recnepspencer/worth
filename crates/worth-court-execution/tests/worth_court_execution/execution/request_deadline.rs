use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[test]
fn request_deadline_observation_preserves_the_earliest_active_ancestor_without_mutating_requests() {
    let early = std::time::Instant::now() + std::time::Duration::from_secs(3600);
    let later = early + std::time::Duration::from_secs(3600);
    let parent = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        Some(early),
    );
    let child = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        Some(later),
    );
    let parent_request = ExecutionRequest::serial(&parent);
    let child_request = ExecutionRequest::serial(&child);
    assert_eq!(child_request.deadline(), Some(later));
    parent_request
        .in_scope(|_| {
            assert_eq!(child_request.deadline(), Some(early));
            child_request
                .in_scope(|_| assert_eq!(child_request.deadline(), Some(early)))
                .unwrap();
        })
        .unwrap();
    assert_eq!(parent_request.deadline(), Some(early));
    assert_eq!(child_request.deadline(), Some(later));
}
