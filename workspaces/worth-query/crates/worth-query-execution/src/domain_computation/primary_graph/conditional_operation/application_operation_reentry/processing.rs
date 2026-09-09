use std::collections::BTreeMap;

use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationSchema, OperationReads, OperationWrites,
    TypedApplicationReadableValue, TypedApplicationValue, WorthQueryInstalledApplicationOperation,
    WorthQueryTemporalIntentRevisionValue, WritableCapability, WritePosture,
};
use worth_runtime_bridge::facade::{BridgeManagedClockBinding, BridgeSealedRuntimeAssembly};

use super::{
    outcome_application::apply_reentry_outcome,
    reenter_temporal_operation,
    settlement_reentry::{self, WorthQuerySettlementReentry},
    wake_retirement::{complete_wake, wake_matches_candidate},
    WorthQueryTemporalReentryCounts,
};
use crate::domain_computation::primary_graph::conditional_operation::{
    operation_invocation::{
        WorthQueryTemporalOperationExecution, WorthQueryTemporalOperationInvoker,
    },
    reconstruction_authority::{
        WorthQueryTemporalPrincipalSource, WorthQueryTemporalReconstructionAccess,
    },
    signal_decision_reentry::{
        WorthQueryRetainedConditionalDecision, WorthQueryRetainedConditionalWake,
    },
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph::conditional_operation) fn reenter_retained_wakes<
    Schema,
    Operation,
    Input,
    Scope,
    PrincipalBinding,
    PrincipalMapping,
    Principal,
    PrincipalIdentity,
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
    bridge: &BridgeSealedRuntimeAssembly,
    clock: &BridgeManagedClockBinding,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    access: &WorthQueryTemporalReconstructionAccess<
        Schema,
        PrincipalBinding,
        PrincipalMapping,
        Principal,
        PrincipalIdentity,
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
    candidates: &mut BTreeMap<
        String,
        super::super::temporal_reconstruction::WorthQueryReconstructedTemporalIntent<Clock, Input>,
    >,
    wakes: &mut [WorthQueryRetainedConditionalWake],
    runtime_binding: &crate::domain_computation::primary_graph::conditional_operation::canonical_identity::WorthQueryTemporalRuntimeBindingIdentity,
) -> WorthQueryTemporalReentryCounts
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
    PrincipalIdentity: worth_query_installation::facade::TypedApplicationIdentityValue,
    ScopeValue: TypedApplicationValue + Clone,
    ScopeWrite: WritePosture,
    ScopeUnit: ApplicationFieldUnit,
    PrincipalSource: WorthQueryTemporalPrincipalSource<Schema>,
    Invoker: WorthQueryTemporalOperationInvoker<Schema, Operation, Input, Scope>,
    IdentityField: OperationReads<Operation>,
    IdentityValue: TypedApplicationReadableValue + Clone,
    IdentityWrite: WritePosture,
    IdentityUnit: ApplicationFieldUnit,
    RevisionField: OperationReads<Operation> + OperationWrites<Operation>,
    RevisionValue: WorthQueryTemporalIntentRevisionValue + TypedApplicationReadableValue + Clone,
    RevisionWrite: WritableCapability,
    RevisionUnit: ApplicationFieldUnit,
    LifecycleField: OperationReads<Operation> + OperationWrites<Operation>,
    LifecycleValue: TypedApplicationReadableValue + Clone,
    LifecycleWrite: WritableCapability,
    LifecycleUnit: ApplicationFieldUnit,
    Authorization:
        super::super::WorthQueryTemporalOperationAuthorization<Schema, Operation, Input, Scope>,
{
    let mut counts = WorthQueryTemporalReentryCounts::default();
    for wake in wakes.iter_mut() {
        wake.application_attempted = false;
        wake.application_admission_canonical_work =
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero();
        let mut settlement_retry = None;
        let decision = std::mem::replace(
            &mut wake.decision,
            WorthQueryRetainedConditionalDecision::Failed(
                "temporal operation re-entry was interrupted".to_string(),
            ),
        );
        let evidence = match decision {
            WorthQueryRetainedConditionalDecision::OperationSettlementDeferred(
                evidence,
                deferred,
            ) => match settlement_reentry::repair(runtime, deferred) {
                WorthQuerySettlementReentry::RetryApplicationPublication(deferred) => {
                    settlement_retry = Some(deferred);
                    evidence
                }
                WorthQuerySettlementReentry::AlreadyCommitted => {
                    complete_wake(
                        bridge,
                        clock,
                        candidates,
                        wake,
                        wake.due.intent_identity().as_str().to_owned(),
                        evidence,
                        &mut counts,
                        false,
                    );
                    continue;
                }
                WorthQuerySettlementReentry::Indeterminate(detail) => {
                    wake.decision = WorthQueryRetainedConditionalDecision::OperationIndeterminate(
                        evidence, detail,
                    );
                    counts.indeterminate += 1;
                    continue;
                }
                WorthQuerySettlementReentry::Deferred(deferred) => {
                    wake.decision =
                        WorthQueryRetainedConditionalDecision::OperationSettlementDeferred(
                            evidence, deferred,
                        );
                    counts.indeterminate += 1;
                    continue;
                }
                WorthQuerySettlementReentry::SnapshotBackpressured(
                    deferred,
                    maximum_active_snapshots,
                ) => {
                    wake.decision =
                        WorthQueryRetainedConditionalDecision::OperationSettlementDeferred(
                            evidence, deferred,
                        );
                    counts.snapshot_capacity_backpressure = Some(maximum_active_snapshots);
                    continue;
                }
                WorthQuerySettlementReentry::RetentionBackpressured(deferred) => {
                    wake.decision =
                        WorthQueryRetainedConditionalDecision::OperationSettlementDeferred(
                            evidence, deferred,
                        );
                    counts.retention_capacity_backpressure = true;
                    continue;
                }
                WorthQuerySettlementReentry::RetentionIdentityExhausted => {
                    wake.decision =
                        WorthQueryRetainedConditionalDecision::OperationTerminalFailure(
                            evidence,
                            super::WorthQueryTemporalTerminalFailure::ApplicationCommit(
                                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::RetentionIdentityExhausted,
                            ),
                        );
                    counts.failed += 1;
                    continue;
                }
                WorthQuerySettlementReentry::SnapshotIdentityExhausted => {
                    wake.decision =
                        WorthQueryRetainedConditionalDecision::OperationTerminalFailure(
                            evidence,
                            super::WorthQueryTemporalTerminalFailure::ApplicationCommit(
                                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::SnapshotIdentityExhausted,
                            ),
                        );
                    counts.failed += 1;
                    continue;
                }
            },
            WorthQueryRetainedConditionalDecision::Eligible(evidence)
            | WorthQueryRetainedConditionalDecision::OperationProductStale(evidence, _)
            | WorthQueryRetainedConditionalDecision::OperationRetryable(evidence, _)
            | WorthQueryRetainedConditionalDecision::OperationBackpressured(evidence, _)
            | WorthQueryRetainedConditionalDecision::OperationIndeterminate(evidence, _) => {
                evidence
            }
            other => {
                wake.decision = other;
                continue;
            }
        };
        let identity = wake.due.intent_identity().as_str();
        let Some(candidate) = candidates.get(identity) else {
            if settlement_retry.is_some() {
                wake.decision = WorthQueryRetainedConditionalDecision::OperationIndeterminate(
                    evidence,
                    "settled publication retry lost its retained temporal candidate".to_string(),
                );
                counts.indeterminate += 1;
            } else {
                wake.decision =
                    WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(evidence);
            }
            continue;
        };
        if !wake_matches_candidate(wake, candidate.candidate()) {
            if settlement_retry.is_some() {
                wake.decision = WorthQueryRetainedConditionalDecision::OperationIndeterminate(
                    evidence,
                    "settled publication retry no longer matches its temporal candidate"
                        .to_string(),
                );
                counts.indeterminate += 1;
            } else {
                wake.decision =
                    WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(evidence);
            }
            continue;
        }
        wake.application_attempted = true;
        let attempt = settlement_retry.map_or_else(
            || {
                reenter_temporal_operation(
                    runtime,
                    operation,
                    access,
                    execution,
                    candidate.candidate(),
                    runtime_binding,
                )
            },
            |deferred| {
                settlement_reentry::retry_application_publication(
                    runtime,
                    candidate.candidate(),
                    runtime_binding,
                    deferred,
                )
            },
        );
        wake.application_admission_canonical_work = attempt.admission_canonical_work;
        apply_reentry_outcome(
            bridge,
            clock,
            candidates,
            wake,
            identity.to_string(),
            evidence,
            attempt.outcome,
            &mut counts,
        );
    }
    counts
}
