use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::{
    PreparedWorkflowAdvance, RequiredWorkflowOperation, WorkflowProgressOutcome,
    WorkflowTransitionRequestKind,
};
use crate::domain_computation::primary_graph::{
    PublishedWorkflowInstanceRef, WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationIdempotencyBinding,
    WorthQueryOperationProjectionDenial, WorthQuerySelectedProductOperation,
};

#[path = "preparation/actor_denial.rs"]
mod actor_denial;
mod adapter_body;
#[path = "preparation/adapter_commit.rs"]
mod adapter_commit;
#[path = "preparation/adapter_owner.rs"]
mod adapter_owner;
#[path = "preparation/adapter_replay.rs"]
mod adapter_replay;
#[path = "preparation/local_actions.rs"]
mod local_actions;
pub use local_actions::validate_action_admission;
mod observation;
/// Which authored local action the binding requests; never a destination or authority.
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::local_actions::WorkflowLocalActionKind;

/// Why a workflow transition could not bind to the branch's program: the installed spec
/// belongs to another schema, the adopted program could not be inspected, the branch runs
/// another program revision, or the branch advanced between selection and read-set
/// assembly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowTransitionBindingDenial {
    ForeignSchema,
    SelectedProgramUnavailable,
    ProgramRevisionChanged,
    SelectedOccurrenceChanged,
}

/// Why a workflow transition did not prepare.
#[derive(Debug)]
pub enum WorkflowTransitionPreparationDenial {
    Binding(WorkflowTransitionBindingDenial),
    Projection(WorthQueryOperationProjectionDenial),
    Attempt(WorthQueryApplicationAttemptDenial),
    IdempotencyInspection(
        crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    ),
}

impl std::fmt::Display for WorkflowTransitionPreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binding(denial) => write!(formatter, "workflow transition binding: {denial:?}"),
            Self::Projection(denial) => denial.fmt(formatter),
            Self::Attempt(denial) => denial.fmt(formatter),
            Self::IdempotencyInspection(denial) => {
                write!(formatter, "workflow settlement custody: {denial:?}")
            }
        }
    }
}

impl std::error::Error for WorkflowTransitionPreparationDenial {}

impl<Schema> WorthQuerySelectedProductOperation<'_, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn prepare_workflow_advance<Capability, Operation, Input, Scope, Spec>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
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
        self.prepare_workflow_transition::<Capability, Operation, Input, Scope, Spec>(
            execution,
            installed,
            instance,
            admission,
            WorkflowTransitionRequestKind::Advance,
        )
    }

    pub fn prepare_workflow_navigate_back<Capability, Operation, Input, Scope, Spec>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
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
        self.prepare_workflow_transition::<Capability, Operation, Input, Scope, Spec>(
            execution,
            installed,
            instance,
            admission,
            WorkflowTransitionRequestKind::NavigateBack,
        )
    }

    pub fn prepare_workflow_collect_assessment<Capability, Operation, Input, Scope, Spec>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
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
        self.prepare_workflow_transition::<Capability, Operation, Input, Scope, Spec>(
            execution,
            installed,
            instance,
            admission,
            WorkflowTransitionRequestKind::CollectAssessment { node_path },
        )
    }

    fn prepare_workflow_transition<Capability, Operation, Input, Scope, Spec>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
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
        let read_set = self.prepare_transition_observation(execution, installed, admission)?;
        read_set
            .authorize_advance::<Capability, Spec>(installed)
            .map_err(WorkflowTransitionPreparationDenial::Attempt)?;
        read_set
            .materialize_admitted_workflow_transition::<Spec>(
                execution,
                installed,
                instance,
                request_kind,
                &self.application().runtime.authorization_clock,
            )
            .map_err(WorkflowTransitionPreparationDenial::Attempt)
    }
}

#[doc(hidden)]
pub struct WorthQueryWorkflowAdvanceAdapter;
