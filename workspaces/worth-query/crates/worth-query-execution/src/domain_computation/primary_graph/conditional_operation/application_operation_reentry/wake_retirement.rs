use std::collections::BTreeMap;
use worth_runtime_bridge::facade::BridgeConditionalExecutionView;

use worth_query_installation::facade::WorthQueryClockCoordinate;
use worth_runtime_bridge::facade::{
    BridgeConditionalDecisionEvidence, BridgeManagedClockBinding,
    BridgeManagedTemporalIntentLifecycle, BridgeManagedTemporalIntentReconciliation,
    BridgeManagedTemporalIntentReconciliationParts,
};

use super::super::{
    operation_invocation::WorthQueryTemporalOccurrenceOutcome,
    signal_decision_reentry::{
        WorthQueryRetainedConditionalDecision, WorthQueryRetainedConditionalWake,
    },
    temporal_reconstruction::WorthQueryReconstructedTemporalIntent,
};
use super::WorthQueryTemporalReentryCounts;

#[allow(clippy::too_many_arguments)]
pub(super) fn complete_wake<Clock, Input>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    bridge: &BridgeConditionalExecutionView<'_>,
    clock: &BridgeManagedClockBinding,
    candidates: &mut BTreeMap<String, WorthQueryReconstructedTemporalIntent<Clock, Input>>,
    wake: &mut WorthQueryRetainedConditionalWake,
    identity: String,
    evidence: BridgeConditionalDecisionEvidence,
    counts: &mut WorthQueryTemporalReentryCounts,
    observed_coordinate: u64,
    rescheduled: Option<WorthQueryTemporalOccurrenceOutcome<Input>>,
    committed: Result<
        Option<worth_foundational::facade::DurableCompositeCommitIdentity>,
        Option<worth_foundational::facade::DurableCompositeCommitIdentity>,
    >,
) {
    if let (
        Ok(_),
        Some(WorthQueryTemporalOccurrenceOutcome::Reschedule {
            next_due_nanoseconds,
            next_input,
            next_input_identity,
            next_idempotency,
        }),
    ) = (&committed, rescheduled)
    {
        let scheduled_due = next_due_nanoseconds.max(observed_coordinate);
        let revision = match wake.due.revision().checked_add(1) {
            Some(revision) => revision,
            None => {
                wake.decision = WorthQueryRetainedConditionalDecision::OperationRetryable(
                    evidence,
                    super::super::WorthQueryConditionalReentryFailure::WakeRevisionExhausted,
                );
                counts.failed += 1;
                return;
            }
        };
        if let Err(detail) = reschedule_committed_wake(
            bridge,
            clock,
            wake,
            revision,
            scheduled_due,
            next_idempotency.as_str(),
        ) {
            wake.decision =
                WorthQueryRetainedConditionalDecision::OperationRetryable(evidence, detail);
            counts.failed += 1;
            return;
        }
        if let Some(existing) = candidates.get(&identity) {
            let successor_bytes =
                std::mem::size_of::<WorthQueryReconstructedTemporalIntent<Clock, Input>>()
                    .saturating_add(existing.candidate().identity().as_str().len())
                    .saturating_add(next_input_identity.as_str().len())
                    .saturating_add(next_idempotency.as_str().len()) as u64;
            if request_execution.consult().is_ok()
                && let Ok(reservation) = request_execution.reserve_retained_memory(successor_bytes)
            {
                let successor_candidate = existing.candidate().rescheduled_successor(
                    revision,
                    WorthQueryClockCoordinate::from_nanoseconds(next_due_nanoseconds),
                    next_input,
                    next_input_identity,
                    next_idempotency,
                );
                let successor =
                    existing.rescheduled(successor_candidate, scheduled_due, reservation);
                candidates.insert(identity, successor);
            } else {
                candidates.remove(&identity);
            }
        }
        if next_due_nanoseconds <= observed_coordinate {
            counts.rescheduled_due_now = true;
        }
    } else {
        if let Err(detail) = retire_committed_wake(bridge, clock, wake) {
            wake.decision =
                WorthQueryRetainedConditionalDecision::OperationRetryable(evidence, detail);
            counts.failed += 1;
            return;
        }
        candidates.remove(&identity);
    }
    match committed {
        Ok(commit) => {
            wake.decision =
                WorthQueryRetainedConditionalDecision::OperationCommitted(evidence, commit);
            counts.committed += 1;
        }
        Err(commit) => {
            wake.decision =
                WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(evidence, commit);
            counts.already_committed += 1;
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn retire_wake_with_decision<Clock, Input>(
    bridge: &BridgeConditionalExecutionView<'_>,
    clock: &BridgeManagedClockBinding,
    candidates: &mut BTreeMap<String, WorthQueryReconstructedTemporalIntent<Clock, Input>>,
    wake: &mut WorthQueryRetainedConditionalWake,
    identity: String,
    evidence: BridgeConditionalDecisionEvidence,
    counts: &mut WorthQueryTemporalReentryCounts,
    decision: impl FnOnce(BridgeConditionalDecisionEvidence) -> WorthQueryRetainedConditionalDecision,
) {
    match retire_committed_wake(bridge, clock, wake) {
        Ok(()) => {
            candidates.remove(&identity);
            wake.decision = decision(evidence);
        }
        Err(detail) => {
            wake.decision =
                WorthQueryRetainedConditionalDecision::OperationRetryable(evidence, detail);
            counts.failed += 1;
        }
    }
}

pub(super) fn wake_matches_candidate<Clock, Input>(
    wake: &WorthQueryRetainedConditionalWake,
    reconstructed: &WorthQueryReconstructedTemporalIntent<Clock, Input>,
) -> bool {
    let candidate = reconstructed.candidate();
    wake.due.revision() == candidate.revision()
        && wake.due.due_coordinate() == reconstructed.scheduled_due_coordinate()
        && wake.due.idempotency_identity() == candidate.idempotency().as_str()
        && wake.due.intent_identity().as_str() == candidate.identity().as_str()
}

fn reschedule_committed_wake(
    bridge: &BridgeConditionalExecutionView<'_>,
    clock: &BridgeManagedClockBinding,
    wake: &WorthQueryRetainedConditionalWake,
    revision: u64,
    scheduled_due: u64,
    idempotency_identity: &str,
) -> Result<(), super::super::WorthQueryConditionalReentryFailure> {
    let outcome = bridge
        .reconcile_managed_temporal_intent(BridgeManagedTemporalIntentReconciliationParts {
            binding: clock,
            identity: wake.due.intent_identity().clone(),
            revision,
            due_coordinate: scheduled_due,
            idempotency_identity: std::sync::Arc::from(idempotency_identity),
            source_record_identity: wake.due.source_record_identity(),
            lifecycle: BridgeManagedTemporalIntentLifecycle::Active,
        })
        .map_err(|denial| {
            super::super::WorthQueryConditionalReentryFailure::ManagedWake(denial.kind())
        })?;
    if matches!(
        outcome,
        BridgeManagedTemporalIntentReconciliation::Superseded
            | BridgeManagedTemporalIntentReconciliation::Installed
            | BridgeManagedTemporalIntentReconciliation::Duplicate
            | BridgeManagedTemporalIntentReconciliation::Stale
    ) {
        Ok(())
    } else {
        Err(super::super::WorthQueryConditionalReentryFailure::WakeDidNotRetire(outcome))
    }
}

fn retire_committed_wake(
    bridge: &BridgeConditionalExecutionView<'_>,
    clock: &BridgeManagedClockBinding,
    wake: &WorthQueryRetainedConditionalWake,
) -> Result<(), super::super::WorthQueryConditionalReentryFailure> {
    let revision = wake
        .due
        .revision()
        .checked_add(1)
        .ok_or(super::super::WorthQueryConditionalReentryFailure::WakeRevisionExhausted)?;
    let outcome = bridge
        .reconcile_managed_temporal_intent(BridgeManagedTemporalIntentReconciliationParts {
            binding: clock,
            identity: wake.due.intent_identity().clone(),
            revision,
            due_coordinate: wake.due.due_coordinate(),
            idempotency_identity: std::sync::Arc::from(wake.due.idempotency_identity()),
            source_record_identity: wake.due.source_record_identity(),
            lifecycle: BridgeManagedTemporalIntentLifecycle::Completed,
        })
        .map_err(|denial| {
            super::super::WorthQueryConditionalReentryFailure::ManagedWake(denial.kind())
        })?;
    if matches!(
        outcome,
        BridgeManagedTemporalIntentReconciliation::Retired
            | BridgeManagedTemporalIntentReconciliation::TerminalNoop
            | BridgeManagedTemporalIntentReconciliation::Stale
    ) {
        Ok(())
    } else {
        Err(super::super::WorthQueryConditionalReentryFailure::WakeDidNotRetire(outcome))
    }
}
