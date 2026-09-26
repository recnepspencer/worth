use super::*;

impl WorthQueryWorkflowAdvanceAdapter {
    pub fn compare_and_commit_assessment<Schema, Operation, Input, Scope, Query>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: super::super::PreparedWorkflowAssessment<Schema, Operation, Input, Scope>,
        settlement: &crate::domain_computation::primary_graph::WorthQueryOutputDemandSettlement,
        source: &crate::domain_computation::primary_graph::WorthQueryObservedSource<Query>,
        posture: crate::domain_computation::primary_graph::WorthQueryWorkflowAssessmentPosture,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorkflowProgressOutcome,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    >
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let prepared = prepared.settle(runtime, settlement, source, posture)?;
        Ok(runtime.compare_and_commit_workflow_advance(prepared, idempotency))
    }

    pub fn compare_and_commit_condition<Schema, Operation, Input, Scope, Binding>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: super::super::PreparedWorkflowCondition<Schema, Operation, Input, Scope>,
        source: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            Binding::Query,
            bool,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorkflowProgressOutcome,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    >
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
        Binding: worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>
            + 'static,
        Binding::Query:
            worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
                    Schema,
                > + 'static,
    {
        let prepared = prepared.settle::<Binding>(runtime, source)?;
        Ok(runtime.compare_and_commit_workflow_advance(prepared, idempotency))
    }

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
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: super::super::PreparedWorkflowOperation<Schema, Operation, Input, Scope>,
        effect_admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<Schema, EffectOperation, EffectInput, EffectScope>,
        effect_idempotency: WorthQueryApplicationIdempotencyBinding,
        required: &RequiredWorkflowOperation,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorkflowProgressOutcome,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    >
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
        EffectInput: Clone + Send + Sync + 'static,
    {
        let custody = runtime
            .resolve_admitted_guarded_workflow_operation_custody(
                effect_admission,
                effect_idempotency,
                required.transition_identity_bytes(),
            )
            .map_err(|_| owner_custody_denial(required))?;
        let crate::domain_computation::primary_graph::application_attempt::WorthQueryGuardedWorkflowOperationCustody::Committed(receipt) = custody else {
            return Err(owner_custody_denial(required));
        };
        let prepared = prepared.settle::<Binding>(runtime, &receipt)?;
        Ok(runtime.compare_and_commit_workflow_advance(prepared, idempotency))
    }

    pub fn compare_and_commit_recovered_operation<
        Schema,
        Operation,
        Input,
        Scope,
        Binding,
        EffectOperation,
        EffectInput,
        EffectScope,
    >(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: super::super::PreparedWorkflowOperation<Schema, Operation, Input, Scope>,
        effect_admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<Schema, EffectOperation, EffectInput, EffectScope>,
        effect_idempotency: WorthQueryApplicationIdempotencyBinding,
        required: &RequiredWorkflowOperation,
        recovery: &crate::domain_computation::application_aftermath::WorthQueryRecoverySafeRetryAdmission,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorkflowProgressOutcome,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    >
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
        EffectInput: Clone + Send + Sync + 'static,
    {
        let custody = runtime
            .resolve_admitted_guarded_workflow_operation_custody(
                effect_admission,
                effect_idempotency,
                required.transition_identity_bytes(),
            )
            .map_err(|_| owner_custody_denial(required))?;
        let crate::domain_computation::primary_graph::application_attempt::WorthQueryGuardedWorkflowOperationCustody::DispatchPending(receipt) = custody else {
            return Err(owner_custody_denial(required));
        };
        let prepared = prepared.settle_recovered::<Binding>(runtime, &receipt, recovery)?;
        Ok(runtime.compare_and_commit_workflow_advance(prepared, idempotency))
    }

    pub fn compare_and_commit<Schema, Operation, Input, Scope>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowProgressOutcome
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        runtime.compare_and_commit_workflow_advance(prepared, idempotency)
    }
}

fn owner_custody_denial(
    required: &RequiredWorkflowOperation,
) -> crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial {
    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        required.node_path(),
    )
}
