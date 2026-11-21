//! `SupportedProgramRecordPort`, `HistoryProducerCutoverPort`, and `WorkflowVocabularyPort` impls.

use std::any::{Any, TypeId};

use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner;
use worth_query_declaration::facade::{
    application_program::{ApplicationProgramRevision, ApplicationWorkflowSpec},
    application_schema::ApplicationSchema,
};
use worth_query_execution_attempt::facade::ports::{
    HistoryProducerCutoverPort, SupportedProgramRecordPort, WorkflowVocabularyPort,
};
use worth_query_installation::facade::{
    WorthQueryInstalledActionInventory, WorthQueryInstalledApplicationWorkflowSpec,
};
use worth_runtime_world::facade::{PreparedWorldRelationalCutover, ProductBranchObservation};

use crate::domain_computation::primary_graph::{
    application_installation::{WorthQuerySupportedProgramRecord, WorthQueryWorkflowVocabulary},
    product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerCutover as LocalHistoryProducerCutover,
};
use crate::equality_type_id::EqualityTypeId;

pub(in crate::domain_computation::primary_graph) fn downcast_supported_program(
    port: &dyn SupportedProgramRecordPort,
) -> &WorthQuerySupportedProgramRecord {
    port.as_any()
        .downcast_ref::<WorthQuerySupportedProgramRecord>()
        .expect("program support carries WorthQuerySupportedProgramRecord")
}

impl SupportedProgramRecordPort for WorthQuerySupportedProgramRecord {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn action_inventory(&self) -> WorthQueryInstalledActionInventory<'_> {
        self.installed.action_inventory()
    }

    fn revision(&self) -> &ApplicationProgramRevision {
        self.revision()
    }

    fn declares_binding(&self, binding: TypeId) -> bool {
        self.declares_action(EqualityTypeId::from(binding))
    }
}

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

impl<Schema, Spec> WorkflowVocabularyPort<Schema, Spec>
    for WorthQueryWorkflowVocabulary<'_, Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    fn workflow_spec_at_revision(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Option<&WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>> {
        self.workflow_spec_at_revision(revision)
    }

    fn authentication_owner(&self) -> &WorthQueryAuthenticationEventSigningOwner<Schema> {
        self.authentication_owner()
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
        approval_authority: Option<&crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis>,
        commit_basis: &crate::domain_computation::authorization::WorthQueryCommitAuthorizationBasis,
        admission: &'admission crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
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
