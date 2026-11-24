use super::super::canonical_identity::WorthQueryTemporalRuntimeBindingIdentity;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use std::collections::BTreeMap;
use worth_runtime_bridge::facade::BridgeConditionalExecutionView;

use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationOperation,
};
use worth_runtime_bridge::facade::BridgeManagedClockBinding;

use super::{
    outcome_application::apply_reentry_outcome, reenter_temporal_operation,
    wake_retirement::wake_matches_candidate, WorthQueryTemporalReentryCounts,
};
use crate::domain_computation::primary_graph::conditional_operation::{
    operation_invocation::{
        WorthQueryTemporalDueFieldWriter, WorthQueryTemporalOperationExecution,
        WorthQueryTemporalOperationInvoker,
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
    ScopeField,
    ScopeValue,
    PrincipalSource,
    QueryAuthorization,
    Invoker,
    IntentEntity,
    IdentityField,
    IdentityValue,
    RevisionField,
    RevisionValue,
    LifecycleField,
    LifecycleValue,
    Authorization,
    DueField,
    Clock,
>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,

    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: &crate::basis::WorthQueryProductBranchLease,
    bridge: &BridgeConditionalExecutionView<'_>,
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
        ScopeField,
        ScopeValue,
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
        IdentityField,
        IdentityValue,
        RevisionField,
        RevisionValue,
        LifecycleField,
        LifecycleValue,
        Authorization,
        DueField,
    >,
    candidates: &mut BTreeMap<
        String,
        super::super::temporal_reconstruction::WorthQueryReconstructedTemporalIntent<Clock, Input>,
    >,
    wakes: &mut [WorthQueryRetainedConditionalWake],
    runtime_binding: &WorthQueryTemporalRuntimeBindingIdentity,
    observed_coordinate: u64,
) -> Result<
    WorthQueryTemporalReentryCounts,
    super::super::super::query_obligations::QueryObligationDenial,
>
where
    Schema: ApplicationSchema,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
    PrincipalIdentity: 'static,
    PrincipalIdentityBinding:
        worth_query_installation::facade::ApplicationIdentityScalarValueBinding<
            Value = PrincipalIdentity,
        >,
    ScopeField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
        Entity = Scope,
        Value = ScopeValue,
    >,
    ScopeField::Binding:
        worth_query_installation::facade::ApplicationScalarValueBinding<Value = ScopeValue>,
    ScopeValue: Clone,
    PrincipalSource: WorthQueryTemporalPrincipalSource<Schema>,
    Invoker: WorthQueryTemporalOperationInvoker<Schema, Operation, Input, Scope>,
    IdentityField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
        Entity = IntentEntity,
        Value = IdentityValue,
    >,
    IdentityField::Binding: worth_query_installation::facade::ApplicationReadableScalarValueBinding<
        Value = IdentityValue,
    >,
    IdentityValue: Clone,
    RevisionField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
        Entity = IntentEntity,
        Value = RevisionValue,
    >,
    RevisionField::Binding: worth_query_installation::facade::ApplicationReadableScalarValueBinding<
            Value = RevisionValue,
        > + worth_query_installation::facade::WorthQueryTemporalIntentRevisionValue,
    RevisionValue: Clone,
    LifecycleField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
        Entity = IntentEntity,
        Value = LifecycleValue,
    >,
    LifecycleField::Binding:
        worth_query_installation::facade::ApplicationReadableScalarValueBinding<
            Value = LifecycleValue,
        >,
    LifecycleValue: Clone,
    Authorization:
        super::super::WorthQueryTemporalOperationAuthorization<Schema, Operation, Input, Scope>,
    DueField: WorthQueryTemporalDueFieldWriter<Schema, IntentEntity>,
{
    let mut counts = WorthQueryTemporalReentryCounts::default();
    for wake in wakes.iter_mut() {
        super::durable_decision::flush_record(runtime, request_execution, wake)?;
    }
    for wake in wakes.iter_mut() {
        // Once one wake commits on this product snapshot, the branch head has
        // moved past `product`; leave remaining eligible wakes untouched so
        // the caller's next step re-selects the fresh branch head cleanly
        // without opening a managed run against a superseded Directory basis.
        if counts.committed > 0 {
            break;
        }
        wake.application_attempted = false;
        wake.application_admission_canonical_work =
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero();
        let decision = std::mem::replace(
            &mut wake.decision,
            WorthQueryRetainedConditionalDecision::InterruptedDuringReentry,
        );
        let was_indeterminate = matches!(
            decision,
            WorthQueryRetainedConditionalDecision::OperationIndeterminate(_, _)
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
            | WorthQueryRetainedConditionalDecision::OperationProductUnpublished(
                _,
                _,
                _,
            )
            | WorthQueryRetainedConditionalDecision::OperationPreparationPending(_, _)
            | WorthQueryRetainedConditionalDecision::OperationPreparationRefused(_, _)
            | WorthQueryRetainedConditionalDecision::OperationCommitted(_, _)
            | WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(_, _)
            | WorthQueryRetainedConditionalDecision::OperationRetiredNoEffect(_)
            | WorthQueryRetainedConditionalDecision::Failed(_)
            | WorthQueryRetainedConditionalDecision::InterruptedDuringReentry) => {
                wake.decision = other;
                continue;
            }
        };
        let identity = wake.due.intent_identity().as_str();
        let candidate_present = candidates
            .get(identity)
            .is_some_and(|candidate| wake_matches_candidate(wake, candidate));
        if !candidate_present {
            let durable_indeterminate = wake.continuation_identity.filter(|cont| {
                was_indeterminate
                    || runtime
                        .installed_program_support()
                        .is_some_and(|support| support.lifecycle().is_indeterminate(&cont.key))
            });
            if let Some(cont) = durable_indeterminate {
                let outcome = super::recovered_original::resolve_live_indeterminate_world_fate(
                    request_execution,
                    runtime,
                    product,
                    cont.key,
                    cont.intent,
                );
                apply_reentry_outcome(
                    bridge,
                    clock,
                    candidates,
                    wake,
                    identity.to_string(),
                    evidence,
                    outcome,
                    observed_coordinate,
                    None,
                    &mut counts,
                );
                super::durable_decision::flush_record(runtime, request_execution, wake)?;
            } else {
                wake.decision = WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(
                    evidence, None,
                );
            }
            continue;
        }
        let candidate = candidates.get(identity).expect("checked candidate above");
        wake.application_attempted = true;
        let mut attempt = match reenter_temporal_operation(
            request_execution,
            phase,
            runtime,
            product,
            operation,
            access,
            execution,
            candidate.candidate(),
            runtime_binding,
        ) {
            Ok(attempt) => attempt,
            Err(cause) => {
                wake.decision = WorthQueryRetainedConditionalDecision::Eligible(evidence);
                return Err(cause);
            }
        };
        if matches!(
            attempt.outcome,
            super::WorthQueryTemporalReentryOutcome::Obsolete
        ) && let Some(cont) = attempt.continuation_identity.filter(|cont| {
            was_indeterminate
                || runtime
                    .installed_program_support()
                    .is_some_and(|support| support.lifecycle().is_indeterminate(&cont.key))
        }) {
            attempt.outcome = super::recovered_original::resolve_live_indeterminate_world_fate(
                request_execution,
                runtime,
                product,
                cont.key,
                cont.intent,
            );
        }
        wake.continuation_identity = attempt.continuation_identity;
        wake.application_admission_canonical_work = attempt.admission_canonical_work;
        apply_reentry_outcome(
            bridge,
            clock,
            candidates,
            wake,
            identity.to_string(),
            evidence,
            attempt.outcome,
            observed_coordinate,
            attempt.rescheduled,
            &mut counts,
        );
        super::durable_decision::flush_record(runtime, request_execution, wake)?;
    }
    Ok(counts)
}
