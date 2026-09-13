mod admitted_projection;
mod attempt_progression;
mod denial;
mod invoker_isolation;
mod outcome_application;
mod processing;
mod provider_commit_deferred;
mod reentry_counts;
mod sealed_commit;
mod settlement_reentry;
mod temporal_idempotency;
mod wake_retirement;
pub(in crate::domain_computation::primary_graph::conditional_operation) use denial::WorthQueryTemporalReentryDenial;
pub(in crate::domain_computation::primary_graph::conditional_operation) use invoker_isolation::isolate_invoker;
pub(super) use processing::reenter_retained_wakes;
pub(super) use reentry_counts::WorthQueryTemporalReentryCounts;
#[cfg(test)]
mod tests;

use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationSchema, OperationReads, OperationWrites,
    WorthQueryInstalledApplicationOperation, WorthQueryTemporalIntentCandidate, WritableCapability,
    WritePosture,
};

use super::operation_invocation::{
    WorthQueryTemporalOperationExecution, WorthQueryTemporalOperationInvoker,
};
use super::reconstruction_authority::{
    WorthQueryTemporalPrincipalSource, WorthQueryTemporalReconstructionAccess,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(super) enum WorthQueryTemporalReentryOutcome {
    ProductStale(crate::domain_computation::WorthQueryProductStaleApplication),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    NoEffect(crate::domain_computation::primary_graph::WorthQueryApplicationNoEffectCause),
    Committed,
    AlreadyCommitted,
    Obsolete,
    RetryableFailure(String),
    SnapshotCapacityBackpressured {
        maximum_active_snapshots: usize,
    },
    RetentionCapacityBackpressured,
    TerminalFailure(WorthQueryTemporalTerminalFailure),
    ProviderCommitBackpressured(
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDeferred,
    ),
    ControlStopped(WorthQueryTemporalControlStop),
    SettlementDeferred(
        crate::domain_computation::primary_graph::WorthQueryApplicationSettlementDeferred,
    ),
    Indeterminate(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorthQueryTemporalControlStop {
    Cancelled,
    TimedOut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorthQueryTemporalTerminalFailure {
    ApplicationCommit(
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind,
    ),
    Admission(WorthQueryTemporalAdmissionTerminalFailure),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorthQueryTemporalAdmissionTerminalFailure {
    Principal(crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenialKind),
    Entity(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind),
    Authorization(
        crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind,
    ),
    Projection(crate::domain_computation::primary_graph::WorthQueryOperationProjectionDenialKind),
    Invariant(crate::domain_computation::primary_graph::WorthQueryInvariantProjectionDenialKind),
}

pub(super) struct WorthQueryTemporalReentryAttempt {
    pub(super) outcome: WorthQueryTemporalReentryOutcome,
    pub(super) admission_canonical_work:
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn reenter_temporal_operation<
    Schema,
    Operation,
    Input,
    Scope,
    PrincipalBinding,
    PrincipalMapping,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
    ScopeAspect,
    ScopeField,
    ScopeValue,
    ScopeWrite,
    ScopeUnit,
    PrincipalSource,
    QueryAuthorization,
    Invoker,
    IntentEntity,
    IdentityAspect,
    IdentityField,
    IdentityValue,
    IdentityWrite,
    IdentityUnit,
    RevisionAspect,
    RevisionField,
    RevisionValue,
    RevisionWrite,
    RevisionEquality,
    RevisionUnit,
    LifecycleAspect,
    LifecycleField,
    LifecycleValue,
    LifecycleWrite,
    LifecycleEquality,
    LifecycleUnit,
    Authorization,
    Clock,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: &crate::basis::WorthQueryProductBranchLease,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    access: &WorthQueryTemporalReconstructionAccess<
        Schema,
        PrincipalBinding,
        PrincipalMapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
        Scope,
        ScopeAspect,
        ScopeField,
        ScopeValue,
        ScopeWrite,
        ScopeUnit,
        PrincipalSource,
        QueryAuthorization,
    >,
    execution: &WorthQueryTemporalOperationExecution<
        Schema,
        Operation,
        Input,
        Scope,
        Invoker,
        IntentEntity,
        IdentityAspect,
        IdentityField,
        IdentityValue,
        IdentityWrite,
        IdentityUnit,
        RevisionAspect,
        RevisionField,
        RevisionValue,
        RevisionWrite,
        RevisionEquality,
        RevisionUnit,
        LifecycleAspect,
        LifecycleField,
        LifecycleValue,
        LifecycleWrite,
        LifecycleEquality,
        LifecycleUnit,
        Authorization,
    >,
    candidate: &WorthQueryTemporalIntentCandidate<Clock, Input>,
    runtime_binding: &super::canonical_identity::WorthQueryTemporalRuntimeBindingIdentity,
) -> WorthQueryTemporalReentryAttempt
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
    PrincipalIdentity: 'static,
    PrincipalIdentityBinding:
        worth_query_installation::facade::ApplicationIdentityScalarValueBinding<
            Value = PrincipalIdentity,
        >,
    ScopeField: worth_query_installation::facade::DeclaredApplicationFieldValue<Value = ScopeValue>,
    ScopeField::Binding:
        worth_query_installation::facade::ApplicationScalarValueBinding<Value = ScopeValue>,
    ScopeValue: Clone,
    ScopeWrite: WritePosture,
    ScopeUnit: ApplicationFieldUnit,
    PrincipalSource: WorthQueryTemporalPrincipalSource<Schema>,
    Invoker: WorthQueryTemporalOperationInvoker<Schema, Operation, Input, Scope>,
    IdentityField: OperationReads<Operation>,
    IdentityField:
        worth_query_installation::facade::DeclaredApplicationFieldValue<Value = IdentityValue>,
    IdentityField::Binding: worth_query_installation::facade::ApplicationReadableScalarValueBinding<
        Value = IdentityValue,
    >,
    IdentityValue: Clone,
    IdentityWrite: WritePosture,
    IdentityUnit: ApplicationFieldUnit,
    RevisionField: OperationReads<Operation> + OperationWrites<Operation>,
    RevisionField:
        worth_query_installation::facade::DeclaredApplicationFieldValue<Value = RevisionValue>,
    RevisionField::Binding: worth_query_installation::facade::ApplicationReadableScalarValueBinding<
            Value = RevisionValue,
        > + worth_query_installation::facade::WorthQueryTemporalIntentRevisionValue,
    RevisionValue: Clone,
    RevisionWrite: WritableCapability,
    RevisionUnit: ApplicationFieldUnit,
    LifecycleField: OperationReads<Operation> + OperationWrites<Operation>,
    LifecycleField:
        worth_query_installation::facade::DeclaredApplicationFieldValue<Value = LifecycleValue>,
    LifecycleField::Binding:
        worth_query_installation::facade::ApplicationReadableScalarValueBinding<
            Value = LifecycleValue,
        >,
    LifecycleValue: Clone,
    LifecycleWrite: WritableCapability,
    LifecycleUnit: ApplicationFieldUnit,
    Authorization: super::WorthQueryTemporalOperationAuthorization<Schema, Operation, Input, Scope>,
{
    let idempotency =
        match temporal_idempotency::prepare_temporal_idempotency(runtime_binding, candidate) {
            Ok(idempotency) => idempotency,
            Err(denial) => {
                return WorthQueryTemporalReentryAttempt {
                    outcome: WorthQueryTemporalReentryOutcome::RetryableFailure(format!(
                        "temporal idempotency admission denied: {denial:?}"
                    )),
                    admission_canonical_work:
                        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
                }
            }
        };
    let admission_canonical_work = idempotency.canonical_work();
    let result = attempt_progression::try_reentry(
        runtime,
        product,
        operation,
        access,
        execution,
        candidate,
        &idempotency,
    );
    let outcome = match result {
        Ok(outcome) => outcome,
        Err(WorthQueryTemporalReentryDenial::Retryable(detail)) => {
            WorthQueryTemporalReentryOutcome::RetryableFailure(detail)
        }
        Err(WorthQueryTemporalReentryDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        }) => WorthQueryTemporalReentryOutcome::SnapshotCapacityBackpressured {
            maximum_active_snapshots,
        },
        Err(WorthQueryTemporalReentryDenial::RetentionCapacityExhausted) => {
            WorthQueryTemporalReentryOutcome::RetentionCapacityBackpressured
        }
        Err(WorthQueryTemporalReentryDenial::ControlStopped(cause)) => {
            WorthQueryTemporalReentryOutcome::ControlStopped(cause)
        }
        Err(WorthQueryTemporalReentryDenial::Terminal(failure)) => {
            WorthQueryTemporalReentryOutcome::TerminalFailure(
                WorthQueryTemporalTerminalFailure::Admission(failure),
            )
        }
    };
    WorthQueryTemporalReentryAttempt {
        outcome,
        admission_canonical_work,
    }
}
