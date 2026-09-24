use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledWorkflowDefinitionContract,
};

use crate::domain_computation::primary_graph::{
    PreparedWorkflowDefinitionPublication, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPreparationDenial, WorkflowDefinitionPublicationOutcome,
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyBinding,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation,
};

/// Narrow cross-crate adapter used by Publication's application-entry owner.
/// It is not a product-facing workflow API.
#[doc(hidden)]
pub struct WorthQueryWorkflowDefinitionPublicationAdapter;

impl WorthQueryWorkflowDefinitionPublicationAdapter {
    #[doc(hidden)]
    pub fn prepare<Schema, Capability, Operation, Input, Scope, Spec, Program>(
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        contract: WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec, Program>,
        expected_predecessor: WorkflowDefinitionExpectedPredecessor,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>,
        WorkflowDefinitionPreparationDenial,
    >
    where
        Schema: ApplicationSchema,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        selected.prepare_workflow_publication::<Capability, Operation, Input, Scope, Spec, Program>(
            contract,
            expected_predecessor,
            admission,
        )
    }

    #[doc(hidden)]
    pub fn compare_and_commit<Schema, Operation, Input, Scope>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowDefinitionPublicationOutcome
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        runtime.compare_and_commit_workflow_definition_publication(prepared, idempotency)
    }
}
