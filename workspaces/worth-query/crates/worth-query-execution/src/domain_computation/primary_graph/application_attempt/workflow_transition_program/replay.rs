//! Replay-resolution adapters for workflow condition and operation progression.

use super::{
    ApplicationMutationBinding, ApplicationSchema, InnerAdapter, PreparedWorkflowAdvance,
    RequiredWorkflowCondition, RequiredWorkflowOperation, WorkflowProgressOutcome,
    WorthQueryAdvancementPhase, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryWorkflowAdvanceAdapter,
    WorthQueryWorkflowConditionSources,
};

impl WorthQueryWorkflowAdvanceAdapter {
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
            phase,
            runtime,
            prepared,
            required,
            sources,
            idempotency,
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
            phase,
            runtime,
            prepared,
            required,
            receipt,
            recovery,
            idempotency,
        )
    }
}
