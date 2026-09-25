use crate::domain_computation::primary_graph::application_installation::WorthQueryWorkflowApplicationRuntime;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::{
    PreparedWorkflowAdvance, RequiredWorkflowApproval, WorkflowApprovalDecision,
    WorkflowProgressOutcome, WorkflowTransitionRequestKind,
};
use crate::domain_computation::primary_graph::{
    PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef,
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationIdempotencyBinding, WorthQueryOperationProjectionDenial,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation,
};

#[path = "preparation/adapter_commit.rs"]
mod adapter_commit;
#[path = "preparation/adapter_preparation.rs"]
mod adapter_preparation;
#[path = "preparation/adapter_replay.rs"]
mod adapter_replay;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowTransitionBindingDenial {
    ForeignSchema,
    SelectedProgramUnavailable,
    ProgramRevisionChanged,
    SelectedOccurrenceChanged,
}

#[derive(Debug)]
pub enum WorkflowTransitionPreparationDenial {
    Binding(WorkflowTransitionBindingDenial),
    Projection(WorthQueryOperationProjectionDenial),
    Attempt(WorthQueryApplicationAttemptDenial),
}

impl std::fmt::Display for WorkflowTransitionPreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binding(denial) => write!(formatter, "workflow transition binding: {denial:?}"),
            Self::Projection(denial) => denial.fmt(formatter),
            Self::Attempt(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for WorkflowTransitionPreparationDenial {}

impl<Schema> WorthQuerySelectedProductOperation<'_, Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn prepare_workflow_advance<
        Capability,
        Operation,
        Input,
        Scope,
        Spec,
        Program,
    >(
        &self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        instance: PublishedWorkflowInstanceRef,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorkflowTransitionPreparationDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.prepare_workflow_transition::<Capability, Operation, Input, Scope, Spec, Program>(
            installed,
            instance,
            admission,
            WorkflowTransitionRequestKind::Advance,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_workflow_navigate_back<
        Capability,
        Operation,
        Input,
        Scope,
        Spec,
        Program,
    >(
        &self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        instance: PublishedWorkflowInstanceRef,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorkflowTransitionPreparationDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.prepare_workflow_transition::<Capability, Operation, Input, Scope, Spec, Program>(
            installed,
            instance,
            admission,
            WorkflowTransitionRequestKind::NavigateBack,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_workflow_collect_assessment<
        Capability,
        Operation,
        Input,
        Scope,
        Spec,
        Program,
    >(
        &self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        instance: PublishedWorkflowInstanceRef,
        node_path: String,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorkflowTransitionPreparationDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.prepare_workflow_transition::<Capability, Operation, Input, Scope, Spec, Program>(
            installed,
            instance,
            admission,
            WorkflowTransitionRequestKind::CollectAssessment { node_path },
        )
    }

    fn prepare_workflow_transition<Capability, Operation, Input, Scope, Spec, Program>(
        &self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        instance: PublishedWorkflowInstanceRef,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        request_kind: WorkflowTransitionRequestKind,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorkflowTransitionPreparationDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if installed.schema_binding() != &self.application().installed_schema().binding_identity() {
            return Err(WorkflowTransitionPreparationDenial::Binding(
                WorkflowTransitionBindingDenial::ForeignSchema,
            ));
        }
        let selected = self.inspect_selected_program().map_err(|_| {
            WorkflowTransitionPreparationDenial::Binding(
                WorkflowTransitionBindingDenial::SelectedProgramUnavailable,
            )
        })?;
        if selected.revision() != installed.program_revision() {
            return Err(WorkflowTransitionPreparationDenial::Binding(
                WorkflowTransitionBindingDenial::ProgramRevisionChanged,
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
            .map_err(WorkflowTransitionPreparationDenial::Projection)?
            .into_parts();
        let read_set = self
            .application()
            .begin_projected_application_read_attempt(admission, projection)
            .map_err(WorkflowTransitionPreparationDenial::Attempt)?
            .complete_projected_dependencies()
            .map_err(WorkflowTransitionPreparationDenial::Attempt)?;
        if selected_occurrence
            != crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                read_set.lease.product().observation(),
            )
        {
            return Err(WorkflowTransitionPreparationDenial::Binding(
                WorkflowTransitionBindingDenial::SelectedOccurrenceChanged,
            ));
        }
        read_set
            .materialize_workflow_advance::<Capability, Spec, Program>(
                installed,
                instance,
                request_kind,
            )
            .map_err(WorkflowTransitionPreparationDenial::Attempt)
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn prepare_workflow_approval<
        Capability,
        Operation,
        Input,
        Scope,
        Spec,
        Program,
    >(
        &self,
        workflow: &WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &PublishedWorkflowProposalRef,
        decision: WorkflowApprovalDecision,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorkflowTransitionPreparationDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let installed = workflow.workflow_spec();
        if installed.schema_binding() != &self.application().installed_schema().binding_identity() {
            return Err(WorkflowTransitionPreparationDenial::Binding(
                WorkflowTransitionBindingDenial::ForeignSchema,
            ));
        }
        let selected = self.inspect_selected_program().map_err(|_| {
            WorkflowTransitionPreparationDenial::Binding(
                WorkflowTransitionBindingDenial::SelectedProgramUnavailable,
            )
        })?;
        if selected.revision() != installed.program_revision() {
            return Err(WorkflowTransitionPreparationDenial::Binding(
                WorkflowTransitionBindingDenial::ProgramRevisionChanged,
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
            .map_err(WorkflowTransitionPreparationDenial::Projection)?
            .into_parts();
        let read_set = self
            .application()
            .begin_projected_application_read_attempt(admission, projection)
            .map_err(WorkflowTransitionPreparationDenial::Attempt)?
            .complete_projected_dependencies()
            .map_err(WorkflowTransitionPreparationDenial::Attempt)?;
        if selected_occurrence
            != crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                read_set.lease.product().observation(),
            )
        {
            return Err(WorkflowTransitionPreparationDenial::Binding(
                WorkflowTransitionBindingDenial::SelectedOccurrenceChanged,
            ));
        }
        read_set
            .materialize_workflow_approval::<Capability, Spec, Program>(
                workflow, instance, required, proposal, decision,
            )
            .map_err(WorkflowTransitionPreparationDenial::Attempt)
    }
}

#[doc(hidden)]
pub struct WorthQueryWorkflowAdvanceAdapter;
