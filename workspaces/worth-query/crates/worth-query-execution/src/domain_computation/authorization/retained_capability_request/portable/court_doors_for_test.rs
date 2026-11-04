//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRetainedCapabilityRequest {
    #[doc(hidden)]
    pub fn decode_workflow_approval_request_for_test(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        encoded: &str,
    ) -> Result<Self, WorkflowApprovalRequestDecodingDenial> {
        Self::decode_workflow_approval_request(execution, encoded)
    }
}
