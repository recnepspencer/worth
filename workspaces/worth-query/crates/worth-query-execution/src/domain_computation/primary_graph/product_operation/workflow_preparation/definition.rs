//! The workflow definition program's publication adapters: definition publication and
//! retirement. Each preparation holds the selection and hands the attempt its
//! `WorkflowSelectionBasis`.
use crate::domain_computation::primary_graph::application_attempt::{OutcomeScopedOps, PublicationScopedOps};
use crate::domain_computation::primary_graph::vocabulary::application_attempt::publication::PublishedWorkflowDefinitionRef;
use crate::domain_computation::primary_graph::{
    PreparedWorkflowDefinitionPublication, PreparedWorkflowDefinitionRetirement,
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPreparationDenial,
    WorkflowDefinitionPublicationOutcome, WorkflowDefinitionRetirementOutcome,
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyBinding,
    WorthQuerySelectedProductOperation,
};
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
    WorthQueryInstalledWorkflowDefinitionContract,
};

mod court_doors_for_test;

/// Narrow cross-crate adapter used by Publication's application-entry owner.
/// It is not a product-facing workflow API.
#[doc(hidden)]
pub struct WorthQueryWorkflowDefinitionPublicationAdapter;

impl WorthQueryWorkflowDefinitionPublicationAdapter {
    #[doc(hidden)]
    pub fn prepare<Schema, Capability, Operation, Input, Scope, Spec>(
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
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
        let phase_request = match phase.execution_request_for(selected.application().product_runtime()) {
            Ok(request) => request,
            Err(_) => return Err(WorkflowDefinitionPreparationDenial::Attempt(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "workflow-phase"))),
        };
        let execution = &phase_request;

        selected.workflow_selection_basis().prepare_workflow_publication::<Capability, Operation, Input, Scope, Spec>(
            execution,
            contract,
            expected_predecessor,
            admission,
        )
    }

    #[doc(hidden)]
    pub fn compare_and_commit<Schema, Operation, Input, Scope>(
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
        prepared: PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowDefinitionPublicationOutcome
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let phase_request = match phase.execution_request_for(runtime.product_runtime()) {
            Ok(request) => request,
            Err(cause) => return WorkflowDefinitionPublicationOutcome::Application(
                crate::domain_computation::request_execution::WorthQueryAdvancementDenial::from(
                    cause,
                )
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

/// Narrow cross-crate adapter used by Publication's application-entry owner.
/// It is not a product-facing workflow API.
#[doc(hidden)]
pub struct WorthQueryWorkflowDefinitionRetirementAdapter;

impl WorthQueryWorkflowDefinitionRetirementAdapter {
    #[doc(hidden)]
    pub fn prepare<Schema, Capability, Operation, Input, Scope, Spec>(
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        definition: PublishedWorkflowDefinitionRef,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowDefinitionRetirement<Schema, Operation, Input, Scope>,
        WorkflowDefinitionPreparationDenial,
    >
    where
        Schema: ApplicationSchema,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let phase_request = match phase.execution_request_for(selected.application().product_runtime()) {
            Ok(request) => request,
            Err(_) => return Err(WorkflowDefinitionPreparationDenial::Attempt(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "workflow-phase"))),
        };
        let execution = &phase_request;

        selected
            .workflow_selection_basis()
            .prepare_workflow_definition_retirement::<Capability, Operation, Input, Scope, Spec>(
                execution, installed, definition, admission,
            )
    }

    #[doc(hidden)]
    pub fn compare_and_commit<Schema, Operation, Input, Scope>(
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
        prepared: PreparedWorkflowDefinitionRetirement<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowDefinitionRetirementOutcome
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let phase_request = match phase.execution_request_for(runtime.product_runtime()) {
            Ok(request) => request,
            Err(cause) => return WorkflowDefinitionRetirementOutcome::Application(
                crate::domain_computation::request_execution::WorthQueryAdvancementDenial::from(
                    cause,
                )
                .into_commit_outcome()
                .landed()
                .expect_err("a foreign phase cannot commit"),
            ),
        };
        let execution = &phase_request;

        runtime.compare_and_commit_workflow_definition_retirement(
            execution,
            phase,
            prepared,
            idempotency,
        )
    }
}
