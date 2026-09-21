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
    WorkflowProgressOutcome,
};
use crate::domain_computation::primary_graph::{
    PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef,
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationIdempotencyBinding, WorthQueryOperationProjectionDenial,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation,
};

#[path = "preparation/adapter_commit.rs"]
mod adapter_commit;

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
            .materialize_workflow_advance::<Capability, Spec, Program>(installed, instance)
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
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
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
                installed, instance, required, proposal, decision,
            )
            .map_err(WorkflowTransitionPreparationDenial::Attempt)
    }
}

#[doc(hidden)]
pub struct WorthQueryWorkflowAdvanceAdapter;

impl WorthQueryWorkflowAdvanceAdapter {
    pub fn requested_instance<Schema, Operation, Input, Scope>(
        prepared: &PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
    ) -> worth_relational::facade::identity::EntityId
    where
        Schema: ApplicationSchema,
        Operation: 'static,
    {
        prepared.requested_instance()
    }

    pub fn resolve_assessment_replay<Schema, Operation, Input, Scope, Query>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: &PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        required: &super::RequiredWorkflowAssessment,
        settlement: &crate::domain_computation::primary_graph::WorthQueryOutputDemandSettlement,
        source: &crate::domain_computation::primary_graph::WorthQueryObservedSource<Query>,
        posture: crate::domain_computation::primary_graph::WorthQueryWorkflowAssessmentPosture,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<WorkflowProgressOutcome>,
        crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    >
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        prepared.resolve_assessment_replay(
            runtime,
            required,
            settlement,
            source,
            posture,
            idempotency,
        )
    }

    #[doc(hidden)]
    pub fn bind_operation_idempotency(
        idempotency: WorthQueryApplicationIdempotencyBinding,
        required: &super::RequiredWorkflowOperation,
    ) -> WorthQueryApplicationIdempotencyBinding {
        idempotency.bind_workflow_operation(required.transition_identity_bytes())
    }

    #[doc(hidden)]
    pub fn bind_operation_idempotency_raw(
        idempotency: WorthQueryApplicationIdempotencyBinding,
        transition_identity: &[u8; 32],
    ) -> WorthQueryApplicationIdempotencyBinding {
        idempotency.bind_workflow_operation(transition_identity)
    }

    pub fn resolve_condition_replay<Schema, Operation, Input, Scope, Query>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: &PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        required: &super::RequiredWorkflowCondition,
        source: &crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            Query,
            bool,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<WorkflowProgressOutcome>,
        crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    >
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        prepared.resolve_condition_replay(runtime, required, source, idempotency)
    }

    pub fn resolve_operation_replay<Schema, Operation, Input, Scope, Binding>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: &PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        required: &super::RequiredWorkflowOperation,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<WorkflowProgressOutcome>,
        crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    >
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
    {
        prepared.resolve_operation_replay::<Binding>(runtime, required, receipt, idempotency)
    }

    pub fn prepare<Schema, Capability, Operation, Input, Scope, Spec, Program>(
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        instance: PublishedWorkflowInstanceRef,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorkflowTransitionPreparationDenial,
    >
    where
        Schema: ApplicationSchema,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        selected.prepare_workflow_advance::<Capability, Operation, Input, Scope, Spec, Program>(
            installed, instance, admission,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare_approval<Schema, Capability, Operation, Input, Scope, Spec, Program>(
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
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
        Schema: ApplicationSchema,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        selected.prepare_workflow_approval::<Capability, Operation, Input, Scope, Spec, Program>(
            installed, instance, required, proposal, decision, admission,
        )
    }
}
