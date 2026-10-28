use super::super::canonical_identity::WorthQueryTemporalRuntimeBindingIdentity;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use std::collections::BTreeMap;

use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationSchema, OperationReads, OperationWrites,
    WorthQueryInstalledApplicationOperation, WritableCapability, WritePosture,
};
use worth_runtime_bridge::facade::{BridgeManagedClockBinding, BridgeSealedRuntimeAssembly};

use super::{
    outcome_application::apply_reentry_outcome, reenter_temporal_operation,
    wake_retirement::wake_matches_candidate, WorthQueryTemporalReentryCounts,
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
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,

    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: &crate::basis::WorthQueryProductBranchLease,
    bridge: &BridgeSealedRuntimeAssembly,
    clock: &BridgeManagedClockBinding,
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
    candidates: &mut BTreeMap<
        String,
        super::super::temporal_reconstruction::WorthQueryReconstructedTemporalIntent<Clock, Input>,
    >,
    wakes: &mut [WorthQueryRetainedConditionalWake],
    runtime_binding: &WorthQueryTemporalRuntimeBindingIdentity,
) -> WorthQueryTemporalReentryCounts
where
    Schema: ApplicationSchema,
    Operation: 'static,
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
    Authorization:
        super::super::WorthQueryTemporalOperationAuthorization<Schema, Operation, Input, Scope>,
{
    let mut counts = WorthQueryTemporalReentryCounts::default();
    for wake in wakes.iter_mut() {
        wake.application_attempted = false;
        wake.application_admission_canonical_work =
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero();
        let decision = std::mem::replace(
            &mut wake.decision,
            WorthQueryRetainedConditionalDecision::InterruptedDuringReentry,
        );
        let evidence = match decision {
            WorthQueryRetainedConditionalDecision::Eligible(evidence)
            | WorthQueryRetainedConditionalDecision::OperationProductStale(evidence, _)
            | WorthQueryRetainedConditionalDecision::OperationNoEffect(evidence, _)
            | WorthQueryRetainedConditionalDecision::OperationCommitRetryable(evidence, _)
            | WorthQueryRetainedConditionalDecision::OperationExecutionControlRetryable(
                evidence,
                _,
            )
            | WorthQueryRetainedConditionalDecision::OperationRetryable(evidence, _)
            | WorthQueryRetainedConditionalDecision::OperationBackpressured(evidence, _)
            | WorthQueryRetainedConditionalDecision::OperationIndeterminate(evidence, _) => {
                evidence
            }
            WorthQueryRetainedConditionalDecision::OperationCleanupRequired(evidence, cleanup) => {
                wake.decision = WorthQueryRetainedConditionalDecision::OperationCleanupRequired(
                    evidence, cleanup,
                );
                counts.indeterminate += 1;
                continue;
            }
            other @ (WorthQueryRetainedConditionalDecision::Suppressed(_)
            | WorthQueryRetainedConditionalDecision::Deferred(_)
            | WorthQueryRetainedConditionalDecision::OperationControlStopped(_, _)
            | WorthQueryRetainedConditionalDecision::OperationTerminalFailure(_, _)
            | WorthQueryRetainedConditionalDecision::OperationProductUnpublished(_, _)
            | WorthQueryRetainedConditionalDecision::OperationCommitted(_)
            | WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(_)
            | WorthQueryRetainedConditionalDecision::Failed(_)
            | WorthQueryRetainedConditionalDecision::InterruptedDuringReentry) => {
                wake.decision = other;
                continue;
            }
        };
        let identity = wake.due.intent_identity().as_str();
        let Some(candidate) = candidates.get(identity) else {
            wake.decision =
                WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(evidence);
            continue;
        };
        if !wake_matches_candidate(wake, candidate.candidate()) {
            wake.decision =
                WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(evidence);
            continue;
        }
        wake.application_attempted = true;
        let attempt = reenter_temporal_operation(
            request_execution,
            phase,
            runtime,
            product,
            operation,
            access,
            execution,
            candidate.candidate(),
            runtime_binding,
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
