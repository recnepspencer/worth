//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::{
    PreparedWorkflowDefinitionPublication, WorkflowDefinitionPreparationDenial,
};
use worth_query_installation::facade::WorthQueryInstalledWorkflowDefinitionContract;

impl<Schema> WorthQuerySelectedProductOperation<'_, Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn prepare_workflow_publication_for_test<Capability, Operation, Input, Scope, Spec>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        contract: WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec>,
        expected_predecessor: crate::domain_computation::primary_graph::WorkflowDefinitionExpectedPredecessor,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>,
        WorkflowDefinitionPreparationDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.workflow_selection_basis()
            .prepare_workflow_publication::<Capability, Operation, Input, Scope, Spec>(
            execution,
            contract,
            expected_predecessor,
            admission,
        )
    }
}
