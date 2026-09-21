use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIntent, ApplicationMutationScopeBinding,
        ApplicationMutationScopeResolution,
    },
    application_program::ApplicationWorkflowSpec,
};
use worth_query_execution::facade::{
    application_installation::WorthQueryWorkflowApplicationRuntime,
    workflow_advance::{
        PreparedWorkflowAdvance, PublishedWorkflowInstanceRef, WorkflowProgressOutcome,
        WorkflowTransitionPreparationDenial, WorthQueryWorkflowAdvanceAdapter,
    },
};
use worth_query_installation::facade::ApplicationSchema;

use crate::application_entry::{
    mutation::{authorization, WorthQueryApplicationMutationRequestWithIdempotency},
    WorthQueryApplicationRequestMutationDenial,
};

type IntentBinding<Schema, Intent> = <Intent as ApplicationMutationIntent<Schema>>::Binding;
type MutationScope<Schema, Binding> =
    <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<
        Schema,
    >>::Scope;
type MutationOperation<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Operation;
type MutationInput<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Input;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowAdvancePreparationDenialKind {
    RuntimeMismatch,
    RequestAdmission,
    TransitionPreparation,
}

#[derive(Debug)]
pub enum WorthQueryWorkflowAdvancePreparationDenial {
    RuntimeMismatch,
    RequestAdmission(WorthQueryApplicationRequestMutationDenial),
    TransitionPreparation(WorkflowTransitionPreparationDenial),
}

#[derive(Debug)]
pub enum WorthQueryWorkflowAssessmentAcceptanceDenial {
    NotAwaitingAssessment,
    RequirementMismatch,
    Replay(
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    ),
    Attempt(worth_query_execution::facade::primary_graph::WorthQueryApplicationAttemptDenial),
}

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
    pub fn prepare_workflow_advance<Spec, Program>(
        mut self,
        workflow: &'application WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>,
        instance: PublishedWorkflowInstanceRef,
    ) -> Result<
        WorthQueryWorkflowAdvanceRequest<
            'application,
            'principal,
            'scope,
            Schema,
            Spec,
            Program,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
        >,
        WorthQueryWorkflowAdvancePreparationDenial,
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Program: worth_query_declaration::facade::application_program::ApplicationProgramDefinition<Schema>,
    {
        let application = self.application_runtime();
        if !std::ptr::eq(application, workflow.program_runtime().runtime()) {
            return Err(WorthQueryWorkflowAdvancePreparationDenial::RuntimeMismatch);
        }
        let selected = application
            .on_branch(self.product_branch())
            .select()
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)
            .map_err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission)?;
        let mutation = authorization::prepare_capability_selected(&mut self, &selected)
            .map_err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission)?;
        let prepared = WorthQueryWorkflowAdvanceAdapter::prepare::<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability,
            MutationOperation<Schema, Intent>,
            MutationInput<Schema, Intent>,
            MutationScope<Schema, IntentBinding<Schema, Intent>>,
            Spec,
            Program,
        >(&selected, workflow.workflow_spec(), instance, mutation.admission)
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

pub struct WorthQueryWorkflowAdvanceRequest<
    'application,
    'principal,
    'scope,
    Schema,
    Spec,
    Program,
    Operation,
    Input,
    Scope,
>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Program: worth_query_declaration::facade::application_program::ApplicationProgramDefinition<Schema>,
{
    pub(super) application: &'application worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) principal: &'principal worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(super) scope: &'scope worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    pub(super) branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    pub(super) workflow: &'application WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>,
    pub(super) prepared: PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
    pub(super) idempotency:
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyBinding,
}

impl<'application, 'principal, 'scope, Schema, Spec, Program, Operation, Input, Scope>
    WorthQueryWorkflowAdvanceRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Spec,
        Program,
        Operation,
        Input,
        Scope,
    >
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Program:
        worth_query_declaration::facade::application_program::ApplicationProgramDefinition<Schema>,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    pub fn into_assessment_demand<Demand>(
        self,
        demand: Demand,
    ) -> Result<
        super::WorthQueryWorkflowAssessmentDemandRequest<
            'application,
            'principal,
            'scope,
            Schema,
            Spec,
            Program,
            Demand,
        >,
        super::WorthQueryWorkflowAssessmentDemandPreparationDenial,
    >
    where
        Demand: worth_query_execution::facade::application_contribution::WorthQueryApplicationOutputDemand<Schema>,
    {
        let required = match self.prepared {
            PreparedWorkflowAdvance::AwaitingAssessment(prepared) => prepared.into_required(),
            PreparedWorkflowAdvance::Transition { .. }
            | PreparedWorkflowAdvance::AwaitingEvidence { .. }
            | PreparedWorkflowAdvance::AwaitingApproval { .. }
            | PreparedWorkflowAdvance::ReplayOnly { .. } => {
                return Err(
                    super::WorthQueryWorkflowAssessmentDemandPreparationDenial::not_assessment(),
                )
            }
        };
        super::WorthQueryWorkflowAssessmentDemandRequest::new(
            self.application,
            self.principal,
            self.scope,
            self.branch,
            self.workflow,
            required,
            demand,
        )
    }

    pub fn execute(self) -> WorkflowProgressOutcome {
        WorthQueryWorkflowAdvanceAdapter::compare_and_commit(
            self.application,
            self.prepared,
            self.idempotency,
        )
    }

    pub fn accept_assessment<Query>(
        self,
        settlement: &super::WorthQueryWorkflowAssessmentDemandSettlement<Query>,
    ) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowAssessmentAcceptanceDenial> {
        if WorthQueryWorkflowAdvanceAdapter::requested_instance(&self.prepared)
            != settlement.required().instance()
        {
            return Err(WorthQueryWorkflowAssessmentAcceptanceDenial::RequirementMismatch);
        }
        if let Some(replayed) = WorthQueryWorkflowAdvanceAdapter::resolve_assessment_replay(
            self.application,
            &self.prepared,
            settlement.required(),
            settlement.owner_settlement().retained(),
            settlement.owner_settlement().observed_source(),
            settlement.posture(),
            self.idempotency,
        )
        .map_err(WorthQueryWorkflowAssessmentAcceptanceDenial::Replay)?
        {
            return Ok(replayed);
        }
        let prepared = match self.prepared {
            PreparedWorkflowAdvance::AwaitingAssessment(prepared) => prepared,
            PreparedWorkflowAdvance::Transition { .. }
            | PreparedWorkflowAdvance::AwaitingEvidence { .. }
            | PreparedWorkflowAdvance::AwaitingApproval { .. }
            | PreparedWorkflowAdvance::ReplayOnly { .. } => {
                return Err(WorthQueryWorkflowAssessmentAcceptanceDenial::NotAwaitingAssessment)
            }
        };
        if !same_requirement(prepared.required(), settlement.required()) {
            return Err(WorthQueryWorkflowAssessmentAcceptanceDenial::RequirementMismatch);
        }
        WorthQueryWorkflowAdvanceAdapter::compare_and_commit_assessment(
            self.application,
            prepared,
            settlement.owner_settlement().retained(),
            settlement.owner_settlement().observed_source(),
            settlement.posture(),
            self.idempotency,
        )
        .map_err(WorthQueryWorkflowAssessmentAcceptanceDenial::Attempt)
    }
}

fn same_requirement(
    left: &worth_query_execution::facade::workflow_advance::RequiredWorkflowAssessment,
    right: &worth_query_execution::facade::workflow_advance::RequiredWorkflowAssessment,
) -> bool {
    left.instance() == right.instance()
        && left.node_path() == right.node_path()
        && left.transition_identity() == right.transition_identity()
        && left.occurrence() == right.occurrence()
        && left.query() == right.query()
        && left.parameter_type() == right.parameter_type()
        && left.result_type() == right.result_type()
        && left.binding() == right.binding()
}
