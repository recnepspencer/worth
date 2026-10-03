use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIntent, ApplicationMutationScopeBinding,
        ApplicationMutationScopeResolution, ApplicationMutationSourceExpectation,
    },
    application_program::ApplicationWorkflowSpec,
};
use worth_query_execution::facade::application_installation::WorthQueryWorkflowVocabulary;
use worth_query_execution::publication_boundary::workflow_advance::{
    PreparedWorkflowAdvance, PublishedWorkflowInstanceRef, RequiredWorkflowAssessment,
    WorkflowProgressOutcome, WorkflowTransitionPreparationDenial, WorthQueryWorkflowAdvanceAdapter,
};
use worth_query_installation::facade::ApplicationSchema;

use crate::application_entry::{
    mutation::{authorization, WorthQueryApplicationMutationRequestWithIdempotency},
    WorthQueryApplicationRequestMutationDenial,
};

#[path = "progress/assessment_affinity.rs"]
mod assessment_affinity;
#[path = "progress/assessment_collection.rs"]
mod assessment_collection;
use assessment_affinity::same_requirement;

pub(super) type IntentBinding<Schema, Intent> =
    <Intent as ApplicationMutationIntent<Schema>>::Binding;
pub(super) type MutationScope<Schema, Binding> =
    <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<
        Schema,
    >>::Scope;
pub(super) type MutationOperation<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Operation;
pub(super) type MutationInput<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Input;

pub(super) type WorkflowAdvancePreparationResult<
    'application,
    'principal,
    'scope,
    Schema,
    Spec,
    Intent,
> = Result<
    WorthQueryWorkflowAdvanceRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Spec,
        MutationOperation<Schema, Intent>,
        MutationInput<Schema, Intent>,
        MutationScope<Schema, IntentBinding<Schema, Intent>>,
    >,
    WorthQueryWorkflowAdvancePreparationDenial,
>;

#[derive(Clone)]
pub(super) enum WorkflowRequestedAction {
    Advance,
    NavigateBack,
    CollectAssessment { node_path: String },
}

/// The kind of a `WorthQueryWorkflowAdvancePreparationDenial`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowAdvancePreparationDenialKind {
    RuntimeMismatch,
    RequestAdmission,
    TransitionPreparation,
    Authentication,
    AwaitingActor,
}

/// Why a workflow advance did not prepare. `RuntimeMismatch` means the workflow belongs to
/// another runtime.
#[derive(Debug)]
pub enum WorthQueryWorkflowAdvancePreparationDenial {
    RuntimeMismatch,
    RequestAdmission(WorthQueryApplicationRequestMutationDenial),
    TransitionPreparation(WorkflowTransitionPreparationDenial),
    Authentication(
        worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial,
    ),
    /// Observation-time readiness; no transition attempt was prepared.
    AwaitingActor(
        worth_query_execution::publication_boundary::workflow_advance::RequiredWorkflowActor,
    ),
}

/// Why `accept_assessment` refused a settled assessment.
#[derive(Debug)]
pub enum WorthQueryWorkflowAssessmentAcceptanceDenial {
    NotAwaitingAssessment,
    RequirementMismatch,
    Replay(
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    ),
    Attempt(worth_query_execution::facade::primary_graph::WorthQueryApplicationAttemptDenial),
}

/// Why a [`WorthQueryWorkflowConditionAcceptance`](super::WorthQueryWorkflowConditionAcceptance)
/// refused its operands.
#[derive(Debug)]
pub enum WorthQueryWorkflowConditionAcceptanceDenial {
    NotAwaitingCondition,
    RequirementMismatch,
    Replay(
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    ),
    Attempt(worth_query_execution::facade::primary_graph::WorthQueryApplicationAttemptDenial),
}

impl std::fmt::Display for WorthQueryWorkflowConditionAcceptanceDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAwaitingCondition => formatter.write_str("workflow is not awaiting condition"),
            Self::RequirementMismatch => formatter
                .write_str("condition result does not match the current workflow requirement"),
            Self::Replay(denial) => denial.fmt(formatter),
            Self::Attempt(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for WorthQueryWorkflowConditionAcceptanceDenial {}

impl std::fmt::Display for WorthQueryWorkflowAssessmentAcceptanceDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAwaitingAssessment => {
                formatter.write_str("workflow is not awaiting assessment")
            }
            Self::RequirementMismatch => formatter
                .write_str("assessment settlement does not match the current workflow requirement"),
            Self::Replay(denial) => denial.fmt(formatter),
            Self::Attempt(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for WorthQueryWorkflowAssessmentAcceptanceDenial {}

impl WorthQueryWorkflowAdvancePreparationDenial {
    pub const fn kind(&self) -> WorthQueryWorkflowAdvancePreparationDenialKind {
        match self {
            Self::RuntimeMismatch => {
                WorthQueryWorkflowAdvancePreparationDenialKind::RuntimeMismatch
            }
            Self::RequestAdmission(_) => {
                WorthQueryWorkflowAdvancePreparationDenialKind::RequestAdmission
            }
            Self::TransitionPreparation(_) => {
                WorthQueryWorkflowAdvancePreparationDenialKind::TransitionPreparation
            }
            Self::Authentication(_) => {
                WorthQueryWorkflowAdvancePreparationDenialKind::Authentication
            }
            Self::AwaitingActor(_) => WorthQueryWorkflowAdvancePreparationDenialKind::AwaitingActor,
        }
    }
}

impl std::fmt::Display for WorthQueryWorkflowAdvancePreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "workflow advance preparation denied: {:?}",
            self.kind()
        )
    }
}

impl std::error::Error for WorthQueryWorkflowAdvancePreparationDenial {}

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
    MutationInput<Schema, Intent>: Clone
        + ApplicationCapabilityRequest<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            Scope = MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
{
    pub fn prepare_workflow_advance<Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'application, Schema, Spec>>,
        instance: PublishedWorkflowInstanceRef,
    ) -> WorkflowAdvancePreparationResult<'application, 'principal, 'scope, Schema, Spec, Intent>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.prepare_workflow_request(workflow.into(), instance, WorkflowRequestedAction::Advance)
    }

    pub(super) fn prepare_workflow_request<Spec>(
        mut self,
        workflow: WorthQueryWorkflowVocabulary<'application, Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        action: WorkflowRequestedAction,
    ) -> WorkflowAdvancePreparationResult<'application, 'principal, 'scope, Schema, Spec, Intent>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let application = self.application_runtime();
        if !std::ptr::eq(application, workflow.runtime()) {
            return Err(WorthQueryWorkflowAdvancePreparationDenial::RuntimeMismatch);
        }
        let selected = application
            .on_branch(self.product_branch())
            .select()
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)
            .map_err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission)?;
        let staged = self.stage().map_err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission)?;
        let identities = self
            .identities()
            .map_err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission)?;
        let mutation = match authorization::prepare_capability_selected(
            &self,
            &identities,
            staged,
            &selected,
        ) {
            Ok(mutation) => mutation,
            Err(denial) => {
                if matches!(action, WorkflowRequestedAction::Advance)
                    && <<IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::SourceExpectation as ApplicationMutationSourceExpectation<Schema>>::QUERY_IDENTIFIER.is_none()
                {
                    if let WorthQueryApplicationRequestMutationDenial::Authorization(actor_denial) = &denial {
                        if instance.branch() == self.product_branch() {
                            let actor = WorthQueryWorkflowAdvanceAdapter::redacted_awaiting_actor::<
                                    <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
                                    MutationOperation<Schema, Intent>,
                                    _, _,
                                >(
                                    workflow.workflow_spec_for(&selected), &instance, actor_denial,
                                );
                            if let Some(actor) = actor {
                                return Err(WorthQueryWorkflowAdvancePreparationDenial::AwaitingActor(actor));
                            }
                        }
                    }
                }
                return Err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission(denial));
            }
        };
        let prepared = match action {
            WorkflowRequestedAction::Advance => WorthQueryWorkflowAdvanceAdapter::prepare::<
                Schema,
                <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
                MutationOperation<Schema, Intent>,
                MutationInput<Schema, Intent>,
                MutationScope<Schema, IntentBinding<Schema, Intent>>,
                Spec,
            >(&selected, workflow.workflow_spec_for(&selected), instance, mutation.admission),
            WorkflowRequestedAction::NavigateBack => WorthQueryWorkflowAdvanceAdapter::prepare_navigate_back::<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
            Spec,
        >(&selected, workflow.workflow_spec_for(&selected), instance, mutation.admission),
            WorkflowRequestedAction::CollectAssessment { node_path } => WorthQueryWorkflowAdvanceAdapter::prepare_collect_assessment::<
                Schema,
                <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
                MutationOperation<Schema, Intent>,
                MutationInput<Schema, Intent>,
                MutationScope<Schema, IntentBinding<Schema, Intent>>,
                Spec,
            >(&selected, workflow.workflow_spec_for(&selected), instance, node_path, mutation.admission),
        }
        .map_err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation)?;
        Ok(WorthQueryWorkflowAdvanceRequest {
            application,
            principal: self.authenticated_principal(),
            scope: self.request_scope(),
            branch: self.product_branch(),
            workflow,
            prepared,
            idempotency: mutation.idempotency,
        })
    }
}

/// A prepared workflow advance. `execute` attempts the transition and reports what the
/// instance needs next.
pub struct WorthQueryWorkflowAdvanceRequest<
    'application,
    'principal,
    'scope,
    Schema,
    Spec,
    Operation,
    Input,
    Scope,
>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub(super) application: &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) principal: &'principal worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(super) scope: &'scope worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    pub(super) branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    pub(super) workflow: WorthQueryWorkflowVocabulary<'application, Schema, Spec>,
    pub(super) prepared: PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
    pub(super) idempotency:
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyBinding,
}

impl<'application, 'principal, 'scope, Schema, Spec, Operation, Input, Scope>
    WorthQueryWorkflowAdvanceRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Spec,
        Operation,
        Input,
        Scope,
    >
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    pub fn execute(self) -> WorkflowProgressOutcome {
        WorthQueryWorkflowAdvanceAdapter::compare_and_commit(
            self.application,
            self.prepared,
            self.idempotency,
        )
    }
}
