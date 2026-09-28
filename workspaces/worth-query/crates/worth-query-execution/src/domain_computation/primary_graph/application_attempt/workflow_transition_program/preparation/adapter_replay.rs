use super::*;

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
        required: &super::super::RequiredWorkflowAssessment,
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
        transition_identity: &[u8; 32],
    ) -> WorthQueryApplicationIdempotencyBinding {
        idempotency.bind_guarded_workflow_effect(transition_identity)
    }

    pub fn resolve_condition_replay<Schema, Operation, Input, Scope>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: &PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        required: &super::super::RequiredWorkflowCondition,
        sources: &crate::domain_computation::primary_graph::WorthQueryWorkflowConditionSources<
            Schema,
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
        prepared.resolve_condition_replay(runtime, required, sources, idempotency)
    }

    pub fn resolve_operation_replay<Schema, Operation, Input, Scope, Binding>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: &PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        required: &super::super::RequiredWorkflowOperation,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        recovery: Option<
            &crate::domain_computation::application_aftermath::WorthQueryRecoverySafeRetryAdmission,
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
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
    {
        prepared.resolve_operation_replay::<Binding>(
            runtime,
            required,
            receipt,
            idempotency,
            recovery,
        )
    }
}
