use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest, ExecutionResourceLease};

/// Fixed-size backing metadata: no rows or index generations are inspected.
pub(crate) fn request_lease<'request, 'authority>(
    execution: &ExecutionRequest<'request, 'authority>,
) -> Option<&'request ExecutionResourceLease<'authority>> {
    match execution.allocation_policy() {
        ExecutionAllocationPolicy::Execution(lease) => Some(lease),
        ExecutionAllocationPolicy::SystemAllocation => None,
    }
}
