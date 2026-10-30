//! The genuine immutable decision payload has a lifetime separate from readers.
use std::sync::Arc;

pub(in crate::conditional_execution) struct BridgeRetainedConditionalObservation {
    pub(in crate::conditional_execution) history:
        crate::diagnostics::conditional_history::ConditionalHistoryReference,
    pub(in crate::conditional_execution) bridge_runtime_key: u64,
    pub(in crate::conditional_execution) lowering_projection:
        super::super::BridgeConditionalLoweringProjectionIdentity,
    pub(in crate::conditional_execution) _definition_retention:
        Option<Arc<super::super::retention::BridgeRetentionReservation>>,
    pub(in crate::conditional_execution) bridge_snapshot_identity:
        Option<crate::snapshot::TruthSnapshotIdentity>,
    pub(in crate::conditional_execution) signal_snapshot_projection: Arc<str>,
    pub(in crate::conditional_execution) signal_execution_projection: Arc<str>,
    pub(in crate::conditional_execution) attempt: u64,
    pub(in crate::conditional_execution) signal:
        worth_signal::facade::SignalConditionalDecisionEvidence,
    pub(in crate::conditional_execution) performed_signal_invalidation:
        Option<worth_signal::facade::adapters::SignalInvalidationCausalObservation>,
    pub(in crate::conditional_execution) semantic_observations:
        super::super::observation_retention::BridgeRetainedObservations,
    pub(in crate::conditional_execution) bridge_execution_counters:
        super::super::BridgeConditionalExecutionCounters,
    pub(in crate::conditional_execution) triggering_change_set:
        Option<super::super::retention::BridgeRetainedTrigger>,
    // This original reservation covers both decision headers. It outlives the
    // operational header when a read-only share remains, conservatively keeping
    // its admitted capacity until the last immutable observation is released.
    pub(in crate::conditional_execution) _reservation:
        super::super::retention::BridgeRetentionReservation,
}
