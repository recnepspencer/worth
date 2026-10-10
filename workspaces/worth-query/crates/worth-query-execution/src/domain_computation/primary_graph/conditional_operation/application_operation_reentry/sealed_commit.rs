use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
    ApplicationSchema, DeclaredApplicationFieldValue, OperationReads, OperationWrites,
    WorthQueryTemporalIntentCandidate, WorthQueryTemporalIntentRevisionValue, WritableCapability,
    WritePosture,
};

use super::{
    admitted_projection::WorthQueryAdmittedTemporalProjection, isolate_invoker,
    temporal_idempotency::WorthQueryPreparedTemporalIdempotency, WorthQueryTemporalReentryOutcome,
};
use crate::domain_computation::primary_graph::conditional_operation::operation_invocation::{
    WorthQueryCurrentTemporalIntent, WorthQueryTemporalOperationExecution,
    WorthQueryTemporalOperationInvoker,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryPrimaryGraphApplicationRuntime,
};

#[rustfmt::skip]
impl<Schema, Operation, Input, Scope, Invoker, IntentEntity, IdentityAspect, IdentityField, IdentityValue, IdentityWrite, IdentityUnit, RevisionAspect, RevisionField, RevisionValue, RevisionWrite, RevisionEquality, RevisionUnit, LifecycleAspect, LifecycleField, LifecycleValue, LifecycleWrite, LifecycleEquality, LifecycleUnit, Authorization>
    WorthQueryTemporalOperationExecution<Schema, Operation, Input, Scope, Invoker, IntentEntity, IdentityAspect, IdentityField, IdentityValue, IdentityWrite, IdentityUnit, RevisionAspect, RevisionField, RevisionValue, RevisionWrite, RevisionEquality, RevisionUnit, LifecycleAspect, LifecycleField, LifecycleValue, LifecycleWrite, LifecycleEquality, LifecycleUnit, Authorization>
where
    Schema: ApplicationSchema,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
    Invoker: WorthQueryTemporalOperationInvoker<Schema, Operation, Input, Scope>,
    IdentityField: DeclaredApplicationFieldValue<Value = IdentityValue>,
    IdentityField::Binding: ApplicationReadableScalarValueBinding<Value = IdentityValue>,
    IdentityWrite: WritePosture,
    IdentityUnit: ApplicationFieldUnit,
    RevisionField: OperationWrites<Operation>
        + DeclaredApplicationFieldValue<Value = RevisionValue>,
    RevisionField::Binding: ApplicationReadableScalarValueBinding<Value = RevisionValue>
        + WorthQueryTemporalIntentRevisionValue,
    RevisionWrite: WritableCapability,
    RevisionUnit: ApplicationFieldUnit,
    LifecycleField: OperationWrites<Operation>,
    LifecycleField: DeclaredApplicationFieldValue<Value = LifecycleValue>,
    LifecycleField::Binding: ApplicationScalarValueBinding<Value = LifecycleValue>,
    LifecycleWrite: WritableCapability,
    LifecycleUnit: ApplicationFieldUnit,
{
    pub(super) fn commit_projected_temporal_effect<Clock>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        candidate: &WorthQueryTemporalIntentCandidate<Clock, Input>,
        current: WorthQueryCurrentTemporalIntent<Schema, IntentEntity, IdentityValue, RevisionValue>,
        projected: WorthQueryAdmittedTemporalProjection<Schema, Operation, Input, Scope, Invoker::Projection>,
        idempotency: &WorthQueryPreparedTemporalIdempotency,
    ) -> Result<WorthQueryTemporalReentryOutcome, super::super::WorthQueryConditionalReentryFailure>
    where
        RevisionField: OperationReads<Operation>,
        RevisionField: DeclaredApplicationFieldValue<Value = RevisionValue>,
        RevisionField::Binding: ApplicationReadableScalarValueBinding<Value = RevisionValue> + worth_query_installation::facade::WorthQueryTemporalIntentRevisionValue,
        RevisionValue: Clone,
        LifecycleField: OperationReads<Operation>,
        LifecycleField::Binding: ApplicationReadableScalarValueBinding<Value = LifecycleValue>,
        LifecycleValue: Clone,
    {
        let reads = runtime
            .begin_projected_application_read_attempt(projected.admission, projected.projection, worth_execution::ExecutionAllocationPolicy::SystemAllocation)
            .map_err(|denial| super::super::WorthQueryConditionalReentryFailure::ApplicationAttempt(denial.kind()))?;
        let mut effects = reads
            .complete_projected_dependencies(crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
            .map_err(|denial| super::super::WorthQueryConditionalReentryFailure::ApplicationAttempt(denial.kind()))?
            .begin_effect_program();
        isolate_invoker(|| {
            self.invoker
                .apply(candidate.input().clone(), projected.host_projection, &mut effects)
        })
        .map_err(|super::invoker_isolation::TemporalInvokerPanicked| super::super::WorthQueryConditionalReentryFailure::EffectInvokerPanicked)?
        .map_err(super::super::WorthQueryConditionalReentryFailure::Invocation)?;
        let target = effects
            .existing_entity(&current.entity)
            .map_err(|denial| super::super::WorthQueryConditionalReentryFailure::ApplicationAttempt(denial.kind()))?;
        let next_revision = candidate
            .revision()
            .checked_add(1)
            .and_then(RevisionField::Binding::from_revision)
            .ok_or(super::super::WorthQueryConditionalReentryFailure::IntentRevisionCannotAdvance)?;
        effects
            .write_field(&target, self.revision_field, next_revision)
            .map_err(|denial| super::super::WorthQueryConditionalReentryFailure::ApplicationAttempt(denial.kind()))?;
        effects
            .write_field(
                &target,
                self.lifecycle_field,
                self.completed_lifecycle.clone(),
            )
            .map_err(|denial| super::super::WorthQueryConditionalReentryFailure::ApplicationAttempt(denial.kind()))?;
        let program = effects.finish().map_err(|denial| super::super::WorthQueryConditionalReentryFailure::ApplicationAttempt(denial.kind()))?;
        Ok(classify_commit(
            runtime.compare_and_commit_conditional_operation(phase, program, idempotency.binding()),
        ))
    }
}

fn classify_commit(
    outcome: WorthQueryApplicationCommitOutcome,
) -> WorthQueryTemporalReentryOutcome {
    match outcome {
        WorthQueryApplicationCommitOutcome::ProductStale(stale) => {
            WorthQueryTemporalReentryOutcome::ProductStale(stale)
        }
        WorthQueryApplicationCommitOutcome::ProductUnpublished(unpublished) => {
            WorthQueryTemporalReentryOutcome::ProductUnpublished(unpublished)
        }
        WorthQueryApplicationCommitOutcome::NoEffect(no_effect) => {
            WorthQueryTemporalReentryOutcome::NoEffect(no_effect.cause())
        }
        WorthQueryApplicationCommitOutcome::Committed(_) => {
            WorthQueryTemporalReentryOutcome::Committed
        }
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(_) => {
            WorthQueryTemporalReentryOutcome::AlreadyCommitted
        }
        WorthQueryApplicationCommitOutcome::Stale(_) => WorthQueryTemporalReentryOutcome::Obsolete,
        WorthQueryApplicationCommitOutcome::Cancelled => {
            WorthQueryTemporalReentryOutcome::ControlStopped(
                super::WorthQueryTemporalControlStop::Cancelled,
            )
        }
        WorthQueryApplicationCommitOutcome::TimedOut => {
            WorthQueryTemporalReentryOutcome::ControlStopped(
                super::WorthQueryTemporalControlStop::TimedOut,
            )
        }
        WorthQueryApplicationCommitOutcome::Denied(denial) => denial::classify_denial(&denial),
        WorthQueryApplicationCommitOutcome::Aborted => {
            WorthQueryTemporalReentryOutcome::RetryableFailure(
                super::super::WorthQueryConditionalReentryFailure::AbortedBeforeEffect,
            )
        }
        WorthQueryApplicationCommitOutcome::Deferred(deferred) => {
            WorthQueryTemporalReentryOutcome::ProviderCommitBackpressured(deferred)
        }
        WorthQueryApplicationCommitOutcome::SettlementDeferred(deferred) => {
            WorthQueryTemporalReentryOutcome::SettlementDeferred(deferred)
        }
        WorthQueryApplicationCommitOutcome::Indeterminate(evidence) => {
            WorthQueryTemporalReentryOutcome::Indeterminate(
                super::super::WorthQueryConditionalReentryFailure::UnresolvedCommit(evidence),
            )
        }
    }
}

mod denial;

#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use denial::tests::assert_preparation_retry;
