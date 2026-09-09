use super::{
    wake_retirement::{complete_wake, retire_obsolete},
    WorthQueryTemporalReentryCounts, WorthQueryTemporalReentryOutcome,
};
use crate::domain_computation::primary_graph::conditional_operation::signal_decision_reentry::{
    WorthQueryRetainedConditionalDecision, WorthQueryRetainedConditionalWake,
};
use std::collections::BTreeMap;
use worth_runtime_bridge::facade::{
    BridgeConditionalDecisionEvidence, BridgeManagedClockBinding, BridgeSealedRuntimeAssembly,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_reentry_outcome<Clock, Input>(
    bridge: &BridgeSealedRuntimeAssembly,
    clock: &BridgeManagedClockBinding,
    candidates: &mut BTreeMap<
        String,
        super::super::temporal_reconstruction::WorthQueryReconstructedTemporalIntent<Clock, Input>,
    >,
    wake: &mut WorthQueryRetainedConditionalWake,
    identity: String,
    evidence: BridgeConditionalDecisionEvidence,
    outcome: WorthQueryTemporalReentryOutcome,
    counts: &mut WorthQueryTemporalReentryCounts,
) {
    match outcome {
        WorthQueryTemporalReentryOutcome::ProductStale(stale) => {
            wake.decision =
                WorthQueryRetainedConditionalDecision::OperationProductStale(evidence, stale);
            counts.failed += 1;
        }
        WorthQueryTemporalReentryOutcome::ProductUnpublished(unpublished) => {
            wake.decision = WorthQueryRetainedConditionalDecision::OperationProductUnpublished(
                evidence,
                unpublished.into_recovery(),
            );
            counts.failed += 1;
        }
        WorthQueryTemporalReentryOutcome::Committed => complete_wake(
            bridge, clock, candidates, wake, identity, evidence, counts, true,
        ),
        WorthQueryTemporalReentryOutcome::AlreadyCommitted => complete_wake(
            bridge, clock, candidates, wake, identity, evidence, counts, false,
        ),
        WorthQueryTemporalReentryOutcome::Obsolete => {
            retire_obsolete(bridge, clock, candidates, wake, identity, evidence, counts)
        }
        WorthQueryTemporalReentryOutcome::RetryableFailure(detail) => {
            wake.decision =
                WorthQueryRetainedConditionalDecision::OperationRetryable(evidence, detail);
            counts.failed += 1;
        }
        WorthQueryTemporalReentryOutcome::SnapshotCapacityBackpressured {
            maximum_active_snapshots,
        } => {
            wake.decision = WorthQueryRetainedConditionalDecision::OperationBackpressured(
                evidence,
                super::super::signal_decision_reentry::WorthQueryOperationBackpressureCause::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                },
            );
            counts.snapshot_capacity_backpressure = Some(maximum_active_snapshots);
        }
        WorthQueryTemporalReentryOutcome::RetentionCapacityBackpressured => {
            wake.decision = WorthQueryRetainedConditionalDecision::OperationBackpressured(
                evidence,
                super::super::signal_decision_reentry::WorthQueryOperationBackpressureCause::RetentionCapacityExhausted,
            );
            counts.retention_capacity_backpressure = true;
        }
        WorthQueryTemporalReentryOutcome::TerminalFailure(kind) => {
            wake.decision =
                WorthQueryRetainedConditionalDecision::OperationTerminalFailure(evidence, kind);
            counts.failed += 1;
        }
        WorthQueryTemporalReentryOutcome::ProviderCommitBackpressured(deferred) => {
            super::provider_commit_deferred::apply_provider_commit_deferred(
                wake, evidence, deferred, counts,
            );
        }
        WorthQueryTemporalReentryOutcome::ControlStopped(cause) => {
            wake.decision =
                WorthQueryRetainedConditionalDecision::OperationControlStopped(evidence, cause);
            counts.failed += 1;
        }
        WorthQueryTemporalReentryOutcome::SettlementDeferred(deferred) => {
            wake.decision = WorthQueryRetainedConditionalDecision::OperationSettlementDeferred(
                evidence, deferred,
            );
            counts.indeterminate += 1;
        }
        WorthQueryTemporalReentryOutcome::SettlementSnapshotCapacityBackpressured {
            deferred,
            maximum_active_snapshots,
        } => {
            wake.decision = WorthQueryRetainedConditionalDecision::OperationSettlementDeferred(
                evidence, deferred,
            );
            counts.snapshot_capacity_backpressure = Some(maximum_active_snapshots);
        }
        WorthQueryTemporalReentryOutcome::Indeterminate(detail) => {
            wake.decision =
                WorthQueryRetainedConditionalDecision::OperationIndeterminate(evidence, detail);
            counts.indeterminate += 1;
        }
    }
}
