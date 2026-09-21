use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::{
    PreparedWorkflowInstanceStart, PublishedWorkflowDefinitionRef, WorkflowInstanceStartOutcome,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationIdempotencyBinding, WorthQueryOperationProjectionDenial,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowInstanceBindingDenial {
    ForeignSchema,
    SelectedProgramUnavailable,
    ProgramRevisionChanged,
    SelectedOccurrenceChanged,
}

#[derive(Debug)]
pub enum WorkflowInstancePreparationDenial {
    Binding(WorkflowInstanceBindingDenial),
    Projection(WorthQueryOperationProjectionDenial),
    Attempt(WorthQueryApplicationAttemptDenial),
}

impl std::fmt::Display for WorkflowInstancePreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binding(denial) => write!(formatter, "workflow instance binding: {denial:?}"),
            Self::Projection(denial) => denial.fmt(formatter),
            Self::Attempt(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for WorkflowInstancePreparationDenial {}

impl<Schema> WorthQuerySelectedProductOperation<'_, Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn prepare_workflow_instance_start<
        Capability,
        Operation,
        Input,
        Scope,
        Spec,
        Program,
    >(
        &self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        published: PublishedWorkflowDefinitionRef,
        start_key_identity: [u8; 32],
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
        WorkflowInstancePreparationDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if installed.schema_binding() != &self.application().installed_schema().binding_identity() {
            return Err(WorkflowInstancePreparationDenial::Binding(
                WorkflowInstanceBindingDenial::ForeignSchema,
            ));
        }
        let selected = self.inspect_selected_program().map_err(|_| {
            WorkflowInstancePreparationDenial::Binding(
                WorkflowInstanceBindingDenial::SelectedProgramUnavailable,
            )
        })?;
        if selected.revision() != installed.program_revision() {
            return Err(WorkflowInstancePreparationDenial::Binding(
                WorkflowInstanceBindingDenial::ProgramRevisionChanged,
            ));
        }
        let selected_occurrence =
            crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                self.product().observation(),
            );
        let (_, projection, _) = self
            .application()
            .mutation_projection
            .project_admitted_operation(&admission, |_, _| {})
            .map_err(WorkflowInstancePreparationDenial::Projection)?
            .into_parts();
        let read_set = self
            .application()
            .begin_projected_application_read_attempt(admission, projection)
            .map_err(WorkflowInstancePreparationDenial::Attempt)?
            .complete_projected_dependencies()
            .map_err(WorkflowInstancePreparationDenial::Attempt)?;
        if selected_occurrence
            != crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                read_set.lease.product().observation(),
            )
        {
            return Err(WorkflowInstancePreparationDenial::Binding(
                WorkflowInstanceBindingDenial::SelectedOccurrenceChanged,
            ));
        }
        read_set
            .materialize_workflow_instance_start::<Capability, Spec, Program>(
                installed,
                published,
                start_key_identity,
            )
            .map_err(WorkflowInstancePreparationDenial::Attempt)
    }
}

#[doc(hidden)]
pub struct WorthQueryWorkflowInstanceStartAdapter;

impl WorthQueryWorkflowInstanceStartAdapter {
    pub fn prepare<Schema, Capability, Operation, Input, Scope, Spec, Program>(
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        published: PublishedWorkflowDefinitionRef,
        start_key_identity: [u8; 32],
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
        WorkflowInstancePreparationDenial,
    >
    where
        Schema: ApplicationSchema,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        selected
            .prepare_workflow_instance_start::<Capability, Operation, Input, Scope, Spec, Program>(
                installed,
                published,
                start_key_identity,
                admission,
            )
    }

    pub fn compare_and_commit<Schema, Operation, Input, Scope>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowInstanceStartOutcome
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        runtime.compare_and_commit_workflow_instance_start(prepared, idempotency)
    }
}
