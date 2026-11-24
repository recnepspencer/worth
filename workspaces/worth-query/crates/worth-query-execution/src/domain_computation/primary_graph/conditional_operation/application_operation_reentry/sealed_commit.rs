use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_execution_attempt::facade::ports::ProjectedReadSetInternalPort as _;
use worth_query_installation::facade::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding, ApplicationSchema,
    WorthQueryTemporalIntentCandidate, WorthQueryTemporalIntentRevisionValue,
};

use super::{
    admitted_projection::WorthQueryAdmittedTemporalProjection, isolate_invoker,
    temporal_idempotency::WorthQueryPreparedTemporalIdempotency, WorthQueryTemporalReentryOutcome,
};
use crate::domain_computation::primary_graph::conditional_operation::operation_invocation::{
    WorthQueryCurrentTemporalIntent, WorthQueryTemporalDueFieldWriter,
    WorthQueryTemporalInvocationFailure, WorthQueryTemporalInvocationFailureKind,
    WorthQueryTemporalOccurrenceOutcome, WorthQueryTemporalOperationExecution,
    WorthQueryTemporalOperationInvoker,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryPrimaryGraphApplicationRuntime,
};

#[rustfmt::skip]
impl<Schema, Operation, Input, Scope, Invoker, IntentEntity, IdentityField, IdentityValue, RevisionField, RevisionValue, LifecycleField, LifecycleValue, Authorization, DueField>
    WorthQueryTemporalOperationExecution<Schema, Operation, Input, Scope, Invoker, IntentEntity, IdentityField, IdentityValue, RevisionField, RevisionValue, LifecycleField, LifecycleValue, Authorization, DueField>
where
    Schema: ApplicationSchema,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
    Invoker: WorthQueryTemporalOperationInvoker<Schema, Operation, Input, Scope>,
    IdentityField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<Entity = IntentEntity, Value = IdentityValue>,
    IdentityField::Binding: ApplicationReadableScalarValueBinding<Value = IdentityValue>,
    RevisionField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<Entity = IntentEntity, Value = RevisionValue>,
    RevisionField::Binding: ApplicationReadableScalarValueBinding<Value = RevisionValue>
        + WorthQueryTemporalIntentRevisionValue,
    LifecycleField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<Entity = IntentEntity, Value = LifecycleValue>,
    LifecycleField::Binding: ApplicationScalarValueBinding<Value = LifecycleValue>,
    DueField: WorthQueryTemporalDueFieldWriter<Schema, IntentEntity>,
{
    pub(super) fn commit_projected_temporal_effect<Clock>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        candidate: &WorthQueryTemporalIntentCandidate<Clock, Input>,
        current: WorthQueryCurrentTemporalIntent<Schema, IntentEntity, IdentityValue, RevisionValue>,
        projected: WorthQueryAdmittedTemporalProjection<Schema, Operation, Input, Scope, Invoker::Projection>,
        idempotency: &WorthQueryPreparedTemporalIdempotency,
    ) -> Result<(WorthQueryTemporalReentryOutcome, Option<WorthQueryTemporalOccurrenceOutcome<Input>>), super::super::WorthQueryConditionalReentryFailure>
    where
        RevisionField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<Entity = IntentEntity, Value = RevisionValue>,
        RevisionField::Binding: ApplicationReadableScalarValueBinding<Value = RevisionValue> + worth_query_installation::facade::WorthQueryTemporalIntentRevisionValue,
        RevisionValue: Clone,
        LifecycleField::Binding: ApplicationReadableScalarValueBinding<Value = LifecycleValue>,
        LifecycleValue: Clone,
    {
        let next_revision_ordinal = candidate
            .revision()
            .checked_add(1)
            .ok_or(super::super::WorthQueryConditionalReentryFailure::IntentRevisionCannotAdvance)?;
        let next_revision = RevisionField::Binding::from_revision(next_revision_ordinal)
            .ok_or(super::super::WorthQueryConditionalReentryFailure::IntentRevisionCannotAdvance)?;
        let occurrence = isolate_invoker(|| {
            self.invoker.next_occurrence(
                candidate.input(),
                &projected.host_projection,
                candidate.due().nanoseconds(),
                next_revision_ordinal,
            )
        })
        .map_err(|super::invoker_isolation::TemporalInvokerPanicked| {
            super::super::WorthQueryConditionalReentryFailure::EffectInvokerPanicked
        })?
        .map_err(super::super::WorthQueryConditionalReentryFailure::Invocation)?;
        if let WorthQueryTemporalOccurrenceOutcome::Reschedule {
            next_due_nanoseconds,
            ..
        } = &occurrence
        {
            if *next_due_nanoseconds <= candidate.due().nanoseconds() {
                return Err(super::super::WorthQueryConditionalReentryFailure::Invocation(
                    WorthQueryTemporalInvocationFailure::new(
                        WorthQueryTemporalInvocationFailureKind::InvocationRejected,
                        "rescheduled temporal intent due coordinate must strictly advance",
                    ),
                ));
            }
            self.due_field
                .validate_reschedule(*next_due_nanoseconds)
                .map_err(super::super::WorthQueryConditionalReentryFailure::Invocation)?;
        }
        let reads = runtime
            .begin_projected_application_read_attempt_with_request(execution, projected.admission, projected.projection, worth_execution::ExecutionAllocationPolicy::SystemAllocation)
            .map_err(|denial| super::super::WorthQueryConditionalReentryFailure::ApplicationAttempt(denial.kind()))?;
        let mut effects = reads
            .complete_projected_dependencies_with_request(execution, crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
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
        effects
            .write_field(&target, self.revision_field, next_revision)
            .map_err(|denial| super::super::WorthQueryConditionalReentryFailure::ApplicationAttempt(denial.kind()))?;
        match &occurrence {
            WorthQueryTemporalOccurrenceOutcome::Complete => {
                effects
                    .write_field(
                        &target,
                        self.lifecycle_field,
                        self.completed_lifecycle.clone(),
                    )
                    .map_err(|denial| super::super::WorthQueryConditionalReentryFailure::ApplicationAttempt(denial.kind()))?;
            }
            WorthQueryTemporalOccurrenceOutcome::Reschedule {
                next_due_nanoseconds,
                ..
            } => {
                effects
                    .write_field(
                        &target,
                        self.lifecycle_field,
                        self.active_lifecycle.clone(),
                    )
                    .map_err(|denial| super::super::WorthQueryConditionalReentryFailure::ApplicationAttempt(denial.kind()))?;
                self.due_field
                    .write_due(&mut effects, &target, *next_due_nanoseconds)?;
            }
        }
        let program = effects.finish().map_err(|denial| super::super::WorthQueryConditionalReentryFailure::ApplicationAttempt(denial.kind()))?;
        let outcome = classify_commit(
            runtime.compare_and_commit_conditional_operation(execution, phase, program, idempotency.binding()),
        );
        let rescheduled = matches!(outcome, WorthQueryTemporalReentryOutcome::Committed(_))
            .then_some(occurrence)
            .filter(|o| matches!(o, WorthQueryTemporalOccurrenceOutcome::Reschedule { .. }));
        Ok((outcome, rescheduled))
    }
}

fn classify_commit(
    outcome: WorthQueryApplicationCommitOutcome,
) -> WorthQueryTemporalReentryOutcome {
    match outcome {
        WorthQueryApplicationCommitOutcome::OutputAdopted(_) => {
            WorthQueryTemporalReentryOutcome::TerminalFailure(
                super::WorthQueryTemporalTerminalFailure::ApplicationCommit(
                    crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::RequiredOutputPreparationMissing))
        }
        WorthQueryApplicationCommitOutcome::ProductPreparationPending(pending) => {
            WorthQueryTemporalReentryOutcome::ProductPreparationPending(pending)
        }
        WorthQueryApplicationCommitOutcome::ProductPreparationRefused(refused) => {
            WorthQueryTemporalReentryOutcome::ProductPreparationRefused(refused)
        }
        WorthQueryApplicationCommitOutcome::CleanupRequired(cleanup) => {
            WorthQueryTemporalReentryOutcome::CleanupRequired(cleanup)
        }
        WorthQueryApplicationCommitOutcome::ProductStale(stale) => {
            WorthQueryTemporalReentryOutcome::ProductStale(stale)
        }
        WorthQueryApplicationCommitOutcome::ProductUnpublished(unpublished) => {
            WorthQueryTemporalReentryOutcome::ProductUnpublished(unpublished)
        }
        WorthQueryApplicationCommitOutcome::NoEffect(no_effect) => {
            WorthQueryTemporalReentryOutcome::NoEffect(no_effect.cause())
        }
        WorthQueryApplicationCommitOutcome::Committed(receipt) => {
            let commit = receipt.committed_product_publication().composite_commit();
            let durable = commit.owner_identity().durable().map(|owner| {
                worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
                    owner,
                    commit.ordinal(),
                )
            });
            WorthQueryTemporalReentryOutcome::Committed(durable)
        }
        WorthQueryApplicationCommitOutcome::RecoveredCommitted(receipt) => {
            WorthQueryTemporalReentryOutcome::AlreadyCommitted(
                receipt
                    .new_product_head()
                    .durable()
                    .map(|head| head.selected_commit()),
            )
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
        WorthQueryApplicationCommitOutcome::Indeterminate(evidence) => {
            WorthQueryTemporalReentryOutcome::Indeterminate(
                super::super::WorthQueryConditionalReentryFailure::UnresolvedCommit(evidence),
            )
        }
    }
}

pub(crate) mod denial;
