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
        required: &super::super::RequiredWorkflowOperation,
    ) -> WorthQueryApplicationIdempotencyBinding {
        idempotency.bind_guarded_workflow_effect(required.transition_identity_bytes())
    }

    #[doc(hidden)]
    pub fn bind_operation_idempotency_raw(
        idempotency: WorthQueryApplicationIdempotencyBinding,
        transition_identity: &[u8; 32],
    ) -> WorthQueryApplicationIdempotencyBinding {
        idempotency.bind_guarded_workflow_effect(transition_identity)
    }

    pub fn resolve_condition_replay<Schema, Operation, Input, Scope, Query>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: &PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        required: &super::super::RequiredWorkflowCondition,
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
        required: &super::super::RequiredWorkflowOperation,
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
        prepared.resolve_operation_replay::<Binding>(runtime, required, receipt, idempotency, None)
    }

    pub fn resolve_recovered_operation_replay<Schema, Operation, Input, Scope, Binding>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: &PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        required: &super::super::RequiredWorkflowOperation,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        recovery: &crate::domain_computation::application_aftermath::WorthQueryRecoverySafeRetryAdmission,
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
            Some(recovery),
        )
    }

    pub fn operation_receipt_requires_recovery(
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> bool {
        super::super::operation::operation_receipt_requires_recovery(receipt)
    }

    #[doc(hidden)]
    pub fn recovery_request_matches_receipt(
        request: WorthQueryApplicationIdempotencyBinding,
        receipt: WorthQueryApplicationIdempotencyBinding,
    ) -> bool {
        receipt.matches_recovery_request(&request)
    }
}
