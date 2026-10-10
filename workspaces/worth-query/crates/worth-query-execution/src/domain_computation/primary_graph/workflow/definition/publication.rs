use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
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
    pub fn prepare<Schema, Capability, Operation, Input, Scope, Spec>(
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        contract: WorthQueryInstalledWorkflowDefinitionContract<Schema, Spec>,
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
        let phase_request = match phase.execution_request_for(&selected.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(WorkflowDefinitionPreparationDenial::Attempt(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "workflow-phase"))),
        };
        let execution = &phase_request;

        selected.prepare_workflow_publication::<Capability, Operation, Input, Scope, Spec>(
            execution,
            contract,
            expected_predecessor,
            admission,
        )
    }

    #[doc(hidden)]
    pub fn compare_and_commit<Schema, Operation, Input, Scope>(
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowDefinitionPublicationOutcome
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let phase_request = match phase.execution_request_for(&runtime.product_runtime) {
            Ok(request) => request,
            Err(_) => return WorkflowDefinitionPublicationOutcome::Application(
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::ForeignPhase
                    .into_commit_outcome()
                    .landed()
                    .expect_err("a foreign phase cannot commit"),
            ),
        };
        let execution = &phase_request;

        runtime.compare_and_commit_workflow_definition_publication(
            execution,
            phase,
            prepared,
            idempotency,
        )
    }
}
