//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn bind_source_partition_for_test(&mut self, identity: [u8; 32]) {
        Self::bind_source_partition(self, identity)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn publication_request_for_test(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        Self::publication_request(self)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn take_authorization_dependencies_for_test(
        &mut self,
        bridge: &worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryProviderCommitAuthorization,
        WorthQueryOperationAuthorizationDenial,
    > {
        Self::take_authorization_dependencies(self, bridge)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn workflow_approval_dependencies_snapshot_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<FundedActorText>, WorkflowApprovalRequestEncodingDenial> {
        Self::workflow_approval_dependencies_snapshot(self, execution)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn workflow_approval_lineage_snapshot_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<FundedActorText>, WorkflowApprovalRequestEncodingDenial> {
        Self::workflow_approval_lineage_snapshot(self, execution)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn workflow_approval_principal_snapshot_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<FundedActorText>, WorkflowApprovalRequestEncodingDenial> {
        Self::workflow_approval_principal_snapshot(self, execution)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn workflow_approval_request_snapshot_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<FundedActorText>, WorkflowApprovalRequestEncodingDenial> {
        Self::workflow_approval_request_snapshot(self, execution)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn workflow_approval_support_snapshot_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<FundedActorText>, WorkflowApprovalRequestEncodingDenial> {
        Self::workflow_approval_support_snapshot(self, execution)
    }
}
