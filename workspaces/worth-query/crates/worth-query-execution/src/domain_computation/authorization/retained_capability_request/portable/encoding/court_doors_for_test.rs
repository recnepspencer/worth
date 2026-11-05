//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::primary_graph::encoding::FundedActorText;

impl WorthQueryRetainedCapabilityRequest {
    #[doc(hidden)]
    pub fn encode_workflow_approval_request_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<FundedActorText, WorkflowApprovalRequestEncodingDenial> {
        Self::encode_workflow_approval_request(self, execution)
    }
}
