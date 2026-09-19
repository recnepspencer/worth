use std::collections::BTreeMap;

use worth_query_installation::facade::WorthQueryTemporalIntentCandidate;
use worth_runtime_bridge::facade::{
    BridgeConditionalDecisionEvidence, BridgeManagedClockBinding,
    BridgeManagedTemporalIntentLifecycle, BridgeManagedTemporalIntentReconciliation,
    BridgeManagedTemporalIntentReconciliationParts, BridgeSealedRuntimeAssembly,
};

use super::super::{
    signal_decision_reentry::{
        WorthQueryRetainedConditionalDecision, WorthQueryRetainedConditionalWake,
    },
    temporal_reconstruction::WorthQueryReconstructedTemporalIntent,
};
use super::WorthQueryTemporalReentryCounts;

#[allow(clippy::too_many_arguments)]
pub(super) fn complete_wake<Clock, Input>(
    bridge: &BridgeSealedRuntimeAssembly,
    clock: &BridgeManagedClockBinding,
    candidates: &mut BTreeMap<String, WorthQueryReconstructedTemporalIntent<Clock, Input>>,
    wake: &mut WorthQueryRetainedConditionalWake,
    identity: String,
    evidence: BridgeConditionalDecisionEvidence,
    counts: &mut WorthQueryTemporalReentryCounts,
    committed: bool,
) {
    if let Err(detail) = retire_committed_wake(bridge, clock, wake) {
        wake.decision = WorthQueryRetainedConditionalDecision::OperationRetryable(evidence, detail);
        counts.failed += 1;
        return;
    }
    candidates.remove(&identity);
    if committed {
        wake.decision = WorthQueryRetainedConditionalDecision::OperationCommitted(evidence);
        counts.committed += 1;
    } else {
        wake.decision = WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(evidence);
        counts.already_committed += 1;
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn retire_obsolete<Clock, Input>(
    bridge: &BridgeSealedRuntimeAssembly,
    clock: &BridgeManagedClockBinding,
    candidates: &mut BTreeMap<String, WorthQueryReconstructedTemporalIntent<Clock, Input>>,
    wake: &mut WorthQueryRetainedConditionalWake,
    identity: String,
    evidence: BridgeConditionalDecisionEvidence,
    counts: &mut WorthQueryTemporalReentryCounts,
) {
    match retire_committed_wake(bridge, clock, wake) {
        Ok(()) => {
            candidates.remove(&identity);
            wake.decision =
                WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(evidence);
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
    candidate: &WorthQueryTemporalIntentCandidate<Clock, Input>,
) -> bool {
    wake.due.revision() == candidate.revision()
        && wake.due.due_coordinate() == candidate.due().nanoseconds()
        && wake.due.idempotency_identity() == candidate.idempotency().as_str()
        && wake.due.intent_identity().as_str() == candidate.identity().as_str()
}

fn retire_committed_wake(
    bridge: &BridgeSealedRuntimeAssembly,
    clock: &BridgeManagedClockBinding,
    wake: &WorthQueryRetainedConditionalWake,
) -> Result<(), String> {
    let revision = wake
        .due
        .revision()
        .checked_add(1)
        .ok_or_else(|| "committed temporal intent revision overflowed".to_string())?;
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
        .map_err(|denial| denial.detail().to_string())?;
    if outcome == BridgeManagedTemporalIntentReconciliation::Retired {
        Ok(())
    } else {
        Err("committed temporal intent did not retire its exact managed wake".to_string())
    }
}
