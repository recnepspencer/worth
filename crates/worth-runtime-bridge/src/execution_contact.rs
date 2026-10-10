//! Admission of one Bridge source or delivery contact under the caller's request.
use crate::error::BridgeExecutionDenial;
use worth_execution::ExecutionRequest;

/// A contact consults the request; the contacted owner charges its own work.
pub(crate) fn admit(request: ExecutionRequest<'_, '_>) -> Result<(), BridgeExecutionDenial> {
    request.consult().map_err(BridgeExecutionDenial::from)
}
