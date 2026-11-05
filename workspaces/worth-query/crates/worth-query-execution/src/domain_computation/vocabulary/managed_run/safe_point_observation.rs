//! Moved whole from `domain_computation::managed_run::safe_point_observation`: the items below and every impl they own.
use std::sync::Arc;
use worth_runtime_bridge::facade::BridgeExecutionQueuePressureState;
use worth_runtime_bridge::facade::BridgeExecutionSafePointCounters;
use worth_runtime_bridge::facade::BridgeExecutionSafePointObservation;
use worth_runtime_bridge::facade::BridgeExecutionSafePointSignalState;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryManagedSafePointObservation {
    run_identity: Arc<str>,
    bridge_evidence: BridgeExecutionSafePointObservation,
}

impl WorthQueryManagedSafePointObservation {
    pub fn run_identity(&self) -> &str {
        &self.run_identity
    }

    pub fn bridge_evidence(&self) -> &BridgeExecutionSafePointObservation {
        &self.bridge_evidence
    }

    pub const fn signal_state(&self) -> BridgeExecutionSafePointSignalState {
        self.bridge_evidence.signal_state()
    }

    pub const fn pressure_state(&self) -> BridgeExecutionQueuePressureState {
        self.bridge_evidence.pressure_state()
    }

    pub const fn queue_depth(&self) -> u64 {
        self.bridge_evidence.queue_depth()
    }

    pub const fn queue_capacity(&self) -> u64 {
        self.bridge_evidence.queue_capacity()
    }

    pub const fn observation_ordinal(&self) -> u64 {
        self.bridge_evidence.observation_ordinal()
    }

    pub const fn lifecycle_ordinal(&self) -> u64 {
        self.bridge_evidence.lifecycle_ordinal()
    }

    pub const fn counters(&self) -> BridgeExecutionSafePointCounters {
        self.bridge_evidence.counters()
    }
}
