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
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        candidate: &WorthQueryTemporalIntentCandidate<Clock, Input>,
        current: WorthQueryCurrentTemporalIntent<Schema, IntentEntity, IdentityValue, RevisionValue>,
        projected: WorthQueryAdmittedTemporalProjection<Schema, Operation, Input, Scope, Invoker::Projection>,
        idempotency: &WorthQueryPreparedTemporalIdempotency,
    ) -> Result<WorthQueryTemporalReentryOutcome, String>
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
            .begin_projected_application_read_attempt(projected.admission, projected.projection)
            .map_err(|denial| denial.to_string())?;
        let mut effects = reads
            .complete_projected_dependencies()
            .map_err(|denial| denial.to_string())?
            .begin_effect_program();
        isolate_invoker(|| {
            self.invoker
                .apply(candidate.input().clone(), projected.host_projection, &mut effects)
        })
        .map_err(|detail| format!("temporal operation invocation failed: {detail}"))?
        .map_err(|failure| format!("{:?}: {}", failure.kind(), failure.detail()))?;
        let target = effects
            .existing_entity(&current.entity)
            .map_err(|denial| denial.to_string())?;
        let next_revision = candidate
            .revision()
            .checked_add(1)
            .and_then(RevisionField::Binding::from_revision)
            .ok_or_else(|| "temporal intent revision cannot advance".to_string())?;
        effects
            .write_field(&target, self.revision_field, next_revision)
            .map_err(|denial| denial.to_string())?;
        effects
            .write_field(
                &target,
                self.lifecycle_field,
                self.completed_lifecycle.clone(),
            )
            .map_err(|denial| denial.to_string())?;
        let program = effects.finish().map_err(|denial| denial.to_string())?;
        Ok(classify_commit(
            runtime.compare_and_commit_application(program, idempotency.binding()),
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
        WorthQueryApplicationCommitOutcome::Denied(denial) => {
            match denial.kind() {
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                } => WorthQueryTemporalReentryOutcome::SnapshotCapacityBackpressured {
                    maximum_active_snapshots,
                },
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::RetentionCapacityExhausted => {
                    WorthQueryTemporalReentryOutcome::RetentionCapacityBackpressured
                }
                kind => WorthQueryTemporalReentryOutcome::TerminalFailure(
                    super::WorthQueryTemporalTerminalFailure::ApplicationCommit(kind),
                ),
            }
        }
        WorthQueryApplicationCommitOutcome::Aborted => {
            WorthQueryTemporalReentryOutcome::RetryableFailure(
                "temporal application commit aborted before effect".to_string(),
            )
        }
        WorthQueryApplicationCommitOutcome::Deferred(deferred) => {
            WorthQueryTemporalReentryOutcome::ProviderCommitBackpressured(deferred)
        }
        WorthQueryApplicationCommitOutcome::SettlementDeferred(deferred) => {
            WorthQueryTemporalReentryOutcome::SettlementDeferred(deferred)
        }
        WorthQueryApplicationCommitOutcome::Indeterminate(evidence) => {
            WorthQueryTemporalReentryOutcome::Indeterminate(evidence.detail().to_string())
        }
    }
}
