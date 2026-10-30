//! Successful execution retains the genuine Native decision and its optional history.
use super::super::retained_decision::BridgeRetainedConditionalDecisionCore;
use super::*;
pub(super) struct RetainedBridgeDecisionOutcome {
    pub(super) history: crate::diagnostics::conditional_history::ConditionalHistoryReference,
    pub(super) reservation: super::super::retention::DecisionReservations,
    pub(super) source_snapshot: Option<
        std::sync::Arc<
            crate::snapshot::AdmittedSnapshotContext<Box<dyn crate::snapshot::TruthSnapshotReader>>,
        >,
    >,
    pub(super) signal: SignalConditionalDecisionEvidence,
    pub(super) observations: super::super::observation_retention::BridgeRetainedObservations,
    pub(super) counters: BridgeConditionalExecutionCounters,
    pub(super) performed_signal_invalidation: Option<SignalInvalidationExecutionReceipt>,
}

pub(super) fn retain_bridge_decision(
    request: &BridgeConditionalExecutionRequest<'_>,
    bridge_runtime_key: u64,
    outcome: RetainedBridgeDecisionOutcome,
) -> BridgeConditionalDecisionEvidence {
    let RetainedBridgeDecisionOutcome {
        history,
        reservation,
        source_snapshot,
        signal,
        observations,
        counters,
        performed_signal_invalidation,
    } = outcome;
    let bridge_snapshot_identity = source_snapshot
        .as_ref()
        .map(|source| source.snapshot_identity().clone());
    let observation = std::sync::Arc::new(
        super::super::causal_observation::BridgeRetainedConditionalObservation {
            _reservation: reservation.core,
            history,
            bridge_runtime_key,
            lowering_projection: request.lowering.projection().clone(),
            _definition_retention: request
                .lowering
                ._definition_retention
                .as_ref()
                .map(std::sync::Arc::clone),
            bridge_snapshot_identity,
            signal_snapshot_projection: request.snapshot_identity.into(),
            signal_execution_projection: request.execution_identity.into(),
            attempt: request.attempt,
            signal,
            performed_signal_invalidation: performed_signal_invalidation
                .as_ref()
                .map(SignalInvalidationExecutionReceipt::retain_causal_observation),
            semantic_observations: observations,
            bridge_execution_counters: counters,
            triggering_change_set: None,
        },
    );
    BridgeConditionalDecisionEvidence {
        _reservation: reservation.evidence,
        core: std::sync::Arc::new(BridgeRetainedConditionalDecisionCore {
            lowering: std::sync::Arc::clone(request.lowering),
            source_snapshot,
            observation,
        }),
        query_binding_identity: request.query_binding_identity.into(),
        query_capability_identity: request.query_capability_identity,
        reentry_counters: BridgeConditionalReentryCounters::default(),
        performed_signal_invalidation,
    }
}
