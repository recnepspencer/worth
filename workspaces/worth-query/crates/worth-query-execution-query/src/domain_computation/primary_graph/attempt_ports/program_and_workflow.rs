//! `HistoryProducerCutoverPort` and `CommitAuthorizationRuntimePort` impls.

use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution_attempt::facade::ports::HistoryProducerCutoverPort;
use worth_runtime_world::facade::{PreparedWorldRelationalCutover, ProductBranchObservation};

use crate::domain_computation::primary_graph::product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerCutover as LocalHistoryProducerCutover;

impl HistoryProducerCutoverPort for LocalHistoryProducerCutover<'_> {
    fn uses_owned_model_graph(&self) -> bool {
        self.uses_owned_model_graph()
    }

    fn current(&self) -> &ProductBranchObservation {
        self.current()
    }

    fn perform(
        &self,
        cutover: PreparedWorldRelationalCutover<'_, '_, '_, '_>,
        expected_target: &ProductBranchObservation,
    ) -> Result<(), ()> {
        (*self).perform(cutover, expected_target).map_err(|_| ())
    }
}

impl<Schema> worth_query_execution_attempt::facade::ports::CommitAuthorizationRuntimePort<Schema>
    for crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    fn authorize_with_commit_basis<'serialization, 'admission, Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        approval_authority: Option<
            &crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis,
        >,
        commit_basis: &crate::domain_computation::authorization::WorthQueryCommitAuthorizationBasis,
        admission: &'admission crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
        serialization: &'serialization crate::domain_computation::primary_graph::WorthQueryApplicationBranchCommitCoordination<'_>,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryApplicationCommitAuthorization<
            'serialization,
            'admission,
            Schema,
            Operation,
            Input,
            Scope,
        >,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    > {
        if let Some(authority) = approval_authority {
            self.readmit_workflow_approval_authority(execution, authority, admission)?;
        }
        self.authorize_application_commit(execution, admission, commit_basis, serialization)
    }
}
