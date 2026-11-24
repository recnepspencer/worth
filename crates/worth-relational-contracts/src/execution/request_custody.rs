use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use worth_execution::ExecutionRequest;

pub fn check_request(request: &ExecutionRequest<'_, '_>) -> Result<(), Cause> {
    request.consult().map_err(Cause::from)
}

use worth_execution::{ExecutionAllocationPolicy, ExecutionResourceLease};

/// Fixed-size backing metadata: no rows or index generations are inspected.
pub fn request_lease<'request, 'authority>(
    execution: &ExecutionRequest<'request, 'authority>,
) -> Option<&'request ExecutionResourceLease<'authority>> {
    match execution.allocation_policy() {
        ExecutionAllocationPolicy::Execution(lease) => Some(lease),
        ExecutionAllocationPolicy::SystemAllocation => None,
    }
}
