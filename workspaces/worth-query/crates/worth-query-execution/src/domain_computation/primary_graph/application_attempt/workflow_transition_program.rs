//! L6 workflow transition adapter combining L6 vocabulary/approval/local-completion
//! operations with thin forwarding onto `worth-query-execution-query`'s adapter.

use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_operation::{
        ApplicationMutationAdmissionBinding, ApplicationMutationBinding,
        ApplicationMutationIdentities, ApplicationMutationScopeBinding,
    },
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_execution_query::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::WorthQueryWorkflowAdvanceAdapter as InnerAdapter;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use crate::domain_computation::primary_graph::application_attempt::{
    PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef,
};
use crate::domain_computation::primary_graph::application_attempt::workflow_transition_program::{
    PreparedWorkflowAdvance, PreparedWorkflowAssessment,
    PreparedWorkflowCondition, PreparedWorkflowOperation, RequiredWorkflowActor,
    RequiredWorkflowApproval, RequiredWorkflowAssessment, RequiredWorkflowCondition,
    RequiredWorkflowOperation, WorkflowApprovalBodyDenial, WorkflowApprovalBodyPreparation,
    WorkflowApprovalDecision, WorkflowLocalActionKind, WorkflowProgressOutcome,
    WorkflowTransitionBindingDenial, WorkflowTransitionPreparationDenial,
    WorthQueryWorkflowActorContext,
};
use crate::domain_computation::primary_graph::application_installation::WorthQueryWorkflowVocabulary;
use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryAdvancementPhase,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationIdempotencyResolutionDenial, WorthQueryObservedSource,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationBodyDecision,
    WorthQueryOperationBodyDenial, WorthQueryOutputDemandSettlement,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryWorkflowAssessmentPosture,
    WorthQueryWorkflowConditionSources,
};

pub struct WorthQueryWorkflowAdvanceAdapter;

#[path = "workflow_transition_program/actor_context.rs"]
mod actor_context;
#[path = "workflow_transition_program/approval.rs"]
mod approval;
#[path = "workflow_transition_program/preparation/adapter_preparation.rs"]
mod adapter_preparation;
#[path = "workflow_transition_program/preparation/local_completion.rs"]
mod local_completion;

impl WorthQueryWorkflowAdvanceAdapter {
    #[doc(hidden)]
    pub fn resolve_guarded_operation_custody<Schema, Operation, Input, Scope>(
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        transition_identity: &[u8; 32],
        completed: Option<&WorthQueryApplicationCommitReceipt>,
    ) -> Result<
        crate::domain_computation::primary_graph::application_attempt::WorthQueryGuardedWorkflowOperationCustody,
        WorthQueryApplicationIdempotencyResolutionDenial,
    >
    where
        Schema: ApplicationSchema,
        Input: Clone + Send + Sync + 'static,
    {
        InnerAdapter::resolve_guarded_operation_custody::<Schema, Operation, Input, Scope>(
            phase,
            runtime,
            admission,
            idempotency,
            transition_identity,
            completed,
        )
    }

    pub fn bind_operation_actor_context<Schema, Spec, Binding>(
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        required: &RequiredWorkflowOperation,
    ) -> Result<Option<WorthQueryWorkflowActorContext>, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Binding: ApplicationMutationBinding<Schema>,
    {
        InnerAdapter::bind_operation_actor_context::<Schema, Spec, Binding>(
            access, installed, required,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare_local_action<S, O, I, Scope, Spec, B>(
        phase: &WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<S, Spec>,
        instance: PublishedWorkflowInstanceRef,
        admission: WorthQueryAdmittedApplicationOperation<S, O, I, Scope>,
        identities: &ApplicationMutationIdentities<'_, S, B>,
        kind: WorkflowLocalActionKind,
    ) -> Result<PreparedWorkflowAdvance<S, O, I, Scope>, WorkflowTransitionPreparationDenial>
    where
        S: ApplicationSchema,
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = S>,
        B: ApplicationMutationAdmissionBinding<S, Operation = O, Input = I>,
        I: 'static,
        O::InputBinding:
            worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
                Value = I,
            >,
    {
        InnerAdapter::prepare_local_action::<S, O, I, Scope, Spec, B>(
            phase, selected, installed, instance, admission, identities, kind,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare_body<Schema, Binding>(
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowAdvance<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        principal_identity: &Binding::PrincipalIdentity,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorkflowApprovalBodyPreparation<
            PreparedWorkflowAdvance<
                Schema,
                Binding::Operation,
                Binding::Input,
                <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            >,
        >,
        WorkflowApprovalBodyDenial,
    >
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<
            Schema,
            Result = (),
            Denial = WorthQueryOperationBodyDenial,
            Decision = WorthQueryOperationBodyDecision<
                Schema,
                <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            >,
        >,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        InnerAdapter::prepare_body::<Schema, Binding>(
            access,
            phase,
            runtime,
            prepared,
            identities,
            principal_identity,
            idempotency,
        )
    }

    pub fn compare_and_commit_assessment<Schema, Operation, Input, Scope, Query>(
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowAssessment<Schema, Operation, Input, Scope>,
        settlement: &WorthQueryOutputDemandSettlement,
        source: &WorthQueryObservedSource<Query>,
        posture: WorthQueryWorkflowAssessmentPosture,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<WorkflowProgressOutcome, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        InnerAdapter::compare_and_commit_assessment::<Schema, Operation, Input, Scope, Query>(
            phase,
            runtime,
            prepared,
            settlement,
            source,
            posture,
            idempotency,
        )
    }

    pub fn compare_and_commit_condition<Schema, Operation, Input, Scope>(
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowCondition<Schema, Operation, Input, Scope>,
        sources: WorthQueryWorkflowConditionSources<Schema>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<WorkflowProgressOutcome, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        InnerAdapter::compare_and_commit_condition::<Schema, Operation, Input, Scope>(
            phase,
            runtime,
            prepared,
            sources,
            idempotency,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compare_and_commit_operation<
        Schema,
        Operation,
        Input,
        Scope,
        Binding,
        EffectOperation,
        EffectInput,
        EffectScope,
    >(
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowOperation<Schema, Operation, Input, Scope>,
        effect_admission: &WorthQueryAdmittedApplicationOperation<
            Schema,
            EffectOperation,
            EffectInput,
            EffectScope,
        >,
        effect_idempotency: WorthQueryApplicationIdempotencyBinding,
        receipt: &WorthQueryApplicationCommitReceipt,
        required: &RequiredWorkflowOperation,
        recovery: Option<
            &crate::domain_computation::application_aftermath::WorthQueryRecoverySafeRetryAdmission,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<WorkflowProgressOutcome, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
        Binding: ApplicationMutationBinding<Schema>,
        EffectInput: Clone + Send + Sync + 'static,
    {
        InnerAdapter::compare_and_commit_operation::<
            Schema,
            Operation,
            Input,
            Scope,
            Binding,
            EffectOperation,
            EffectInput,
            EffectScope,
        >(
            phase,
            runtime,
            prepared,
            effect_admission,
            effect_idempotency,
            receipt,
            required,
            recovery,
            idempotency,
        )
    }

    pub fn compare_and_commit<Schema, Operation, Input, Scope>(
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowProgressOutcome
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        InnerAdapter::compare_and_commit::<Schema, Operation, Input, Scope>(
            phase,
            runtime,
            prepared,
            idempotency,
        )
    }

    pub fn redacted_awaiting_actor<Capability, Operation, Schema, Spec>(
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        instance: &PublishedWorkflowInstanceRef,
        denial: &WorthQueryOperationAuthorizationDenial,
    ) -> Option<RequiredWorkflowActor>
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        InnerAdapter::redacted_awaiting_actor::<Capability, Operation, Schema, Spec>(
            installed, instance, denial,
        )
    }

    pub fn requested_instance<Schema, Operation, Input, Scope>(
        prepared: &PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
    ) -> worth_relational::facade::identity::EntityId
    where
        Schema: ApplicationSchema,
        Operation: 'static,
    {
        InnerAdapter::requested_instance::<Schema, Operation, Input, Scope>(prepared)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn resolve_assessment_replay<Schema, Operation, Input, Scope, Query>(
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: &PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        required: &RequiredWorkflowAssessment,
        settlement: &WorthQueryOutputDemandSettlement,
        source: &WorthQueryObservedSource<Query>,
        posture: WorthQueryWorkflowAssessmentPosture,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<Option<WorkflowProgressOutcome>, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        InnerAdapter::resolve_assessment_replay::<Schema, Operation, Input, Scope, Query>(
            phase,
            runtime,
            prepared,
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
        transition_identity: &[u8; 32],
    ) -> WorthQueryApplicationIdempotencyBinding {
        InnerAdapter::bind_operation_idempotency(idempotency, transition_identity)
    }

    pub fn resolve_condition_replay<Schema, Operation, Input, Scope>(
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: &PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        required: &RequiredWorkflowCondition,
        sources: &WorthQueryWorkflowConditionSources<Schema>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<Option<WorkflowProgressOutcome>, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        InnerAdapter::resolve_condition_replay::<Schema, Operation, Input, Scope>(
            phase, runtime, prepared, required, sources, idempotency,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn resolve_operation_replay<Schema, Operation, Input, Scope, Binding>(
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: &PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        required: &RequiredWorkflowOperation,
        receipt: &WorthQueryApplicationCommitReceipt,
        recovery: Option<
            &crate::domain_computation::application_aftermath::WorthQueryRecoverySafeRetryAdmission,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<Option<WorkflowProgressOutcome>, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
        Binding: ApplicationMutationBinding<Schema>,
    {
        InnerAdapter::resolve_operation_replay::<Schema, Operation, Input, Scope, Binding>(
            phase, runtime, prepared, required, receipt, recovery, idempotency,
        )
    }
}
