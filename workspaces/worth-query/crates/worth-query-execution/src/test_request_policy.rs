//! Named serial host policy for direct primitive fixture controls.
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

pub(crate) fn serial_request() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    )
}

/// Allocation probes narrow the invoking request instead of opening a foreign root.
pub(crate) fn memory_child<'authority>(
    request: worth_execution::ExecutionRequest<'_, 'authority>,
    bytes: u64,
) -> worth_execution::ExecutionResourceLease<'authority> {
    request
        .in_scope(|parent| {
            let parent = parent.expect("the installed phase owns a lease");
            parent
                .child(worth_execution::LeaseRequest {
                    policy: worth_foundational::ExecutionRequestPolicy::new(
                        parent.policy().posture(),
                        parent.policy().determinism(),
                        worth_foundational::ExecutionBudget::new(
                            parent.policy().max_workers(),
                            Some(bytes),
                            parent.policy().work_ceiling(),
                        ),
                    ),
                    cancellation: worth_execution::CancellationToken::new(),
                    deadline: None,
                })
                .unwrap()
        })
        .unwrap()
}
