use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;

impl WorthQueryWorkflowAdvanceAdapter {
    pub fn compare_and_commit_assessment<Schema, Operation, Input, Scope, Query>(
        phase: &WorthQueryAdvancementPhase<'_>,

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
        if let Err(cause) = phase.execution_request_for(&runtime.product_runtime) {
            let uncommitted =
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(cause)
                    .into_commit_outcome()
                    .landed()
                    .expect_err("a foreign phase cannot commit");
            return Ok(WorkflowProgressOutcome::Application(uncommitted));
        }
        let prepared = prepared.settle(runtime, settlement, source, posture)?;
        Ok(runtime.compare_and_commit_workflow_advance(phase, prepared, idempotency))
    }

    pub fn compare_and_commit_condition<Schema, Operation, Input, Scope>(
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: super::super::PreparedWorkflowCondition<Schema, Operation, Input, Scope>,
        sources: crate::domain_computation::primary_graph::WorthQueryWorkflowConditionSources<
            Schema,
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
    {
        if let Err(cause) = phase.execution_request_for(&runtime.product_runtime) {
            let uncommitted =
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(cause)
                    .into_commit_outcome()
                    .landed()
                    .expect_err("a foreign phase cannot commit");
            return Ok(WorkflowProgressOutcome::Application(uncommitted));
        }
        let prepared = prepared.settle(runtime, sources)?;
        Ok(runtime.compare_and_commit_workflow_advance(phase, prepared, idempotency))
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
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: super::super::PreparedWorkflowOperation<Schema, Operation, Input, Scope>,
        effect_admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<Schema, EffectOperation, EffectInput, EffectScope>,
        effect_idempotency: WorthQueryApplicationIdempotencyBinding,
        required: &RequiredWorkflowOperation,
        recovery: Option<
            &crate::domain_computation::application_aftermath::WorthQueryRecoverySafeRetryAdmission,
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
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
        EffectInput: Clone + Send + Sync + 'static,
    {
        if let Err(cause) = phase.execution_request_for(&runtime.product_runtime) {
            let uncommitted =
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(cause)
                    .into_commit_outcome()
                    .landed()
                    .expect_err("a foreign phase cannot commit");
            return Ok(WorkflowProgressOutcome::Application(uncommitted));
        }
        let custody = runtime
            .resolve_admitted_guarded_workflow_operation_custody(
                effect_admission,
                effect_idempotency,
                required.transition_identity_bytes(),
            )
            .map_err(|_| owner_custody_denial(required))?;
        use crate::domain_computation::primary_graph::application_attempt::WorthQueryGuardedWorkflowOperationCustody as Custody;
        // An owner-settled external effect has the same original operation
        // receipt. Receipt validation rechecks its canonical terminal; a
        // dispatch-pending effect still requires exact safe-retry admission.
        let prepared = match (custody, recovery) {
            (Custody::Committed(receipt), None) => prepared.settle::<Binding>(runtime, &receipt)?,
            (Custody::ExternallySettled(receipt), _) => {
                prepared.settle::<Binding>(runtime, &receipt)?
            }
            (Custody::DispatchPending(receipt), Some(recovery)) => {
                prepared.settle_recovered::<Binding>(runtime, &receipt, recovery)?
            }
            _ => return Err(owner_custody_denial(required)),
        };
        Ok(runtime.compare_and_commit_workflow_advance(phase, prepared, idempotency))
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
        runtime.compare_and_commit_workflow_advance(phase, prepared, idempotency)
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
