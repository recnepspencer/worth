//! Definition retirement preparation for one selected branch occurrence.

use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::{WorkflowDefinitionBindingDenial, WorkflowDefinitionPreparationDenial};
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::{
    PreparedWorkflowDefinitionRetirement, PublishedWorkflowDefinitionRef,
    WorkflowDefinitionRetirementOutcome, WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationIdempotencyBinding, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQuerySelectedProductOperation,
};

impl<Schema> WorthQuerySelectedProductOperation<'_, Schema>
where
    Schema: ApplicationSchema,
{
    fn prepare_workflow_definition_retirement<Capability, Operation, Input, Scope, Spec, Program>(
        &self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        definition: PublishedWorkflowDefinitionRef,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowDefinitionRetirement<Schema, Operation, Input, Scope>,
        WorkflowDefinitionPreparationDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if installed.schema_binding() != &self.application().installed_schema().binding_identity() {
            return Err(WorkflowDefinitionPreparationDenial::Binding(
                WorkflowDefinitionBindingDenial::ForeignSchema,
            ));
        }
        let selected = self.inspect_selected_program().map_err(|_| {
            WorkflowDefinitionPreparationDenial::Binding(
                WorkflowDefinitionBindingDenial::SelectedProgramUnavailable,
            )
        })?;
        if selected.revision() != installed.program_revision() {
            return Err(WorkflowDefinitionPreparationDenial::Binding(
                WorkflowDefinitionBindingDenial::ProgramRevisionChanged,
            ));
        }
        let selected_occurrence =
            WorthQueryProductBranchReadIdentity::from_observation(self.product().observation());
        let (_, projection, _) = self
            .application()
            .mutation_projection
            .project_admitted_operation(&admission, |_, _| {})
            .map_err(WorkflowDefinitionPreparationDenial::Projection)?
            .into_parts();
        let read_set = self
            .application()
            .begin_projected_application_read_attempt(admission, projection)
            .map_err(WorkflowDefinitionPreparationDenial::Attempt)?
            .complete_projected_dependencies()
            .map_err(WorkflowDefinitionPreparationDenial::Attempt)?;
        read_set
            .materialize_workflow_definition_retirement::<Capability, Spec, Program>(
                installed,
                definition,
                &selected_occurrence,
            )
            .map_err(WorkflowDefinitionPreparationDenial::Attempt)
    }
}

/// Narrow cross-crate adapter used by Publication's application-entry owner.
/// It is not a product-facing workflow API.
#[doc(hidden)]
pub struct WorthQueryWorkflowDefinitionRetirementAdapter;

impl WorthQueryWorkflowDefinitionRetirementAdapter {
    #[doc(hidden)]
    pub fn prepare<Schema, Capability, Operation, Input, Scope, Spec, Program>(
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
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
        selected
            .prepare_workflow_definition_retirement::<Capability, Operation, Input, Scope, Spec, Program>(
                installed, definition, admission,
            )
    }

    #[doc(hidden)]
    pub fn compare_and_commit<Schema, Operation, Input, Scope>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowDefinitionRetirement<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowDefinitionRetirementOutcome
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        runtime.compare_and_commit_workflow_definition_retirement(prepared, idempotency)
    }
}
