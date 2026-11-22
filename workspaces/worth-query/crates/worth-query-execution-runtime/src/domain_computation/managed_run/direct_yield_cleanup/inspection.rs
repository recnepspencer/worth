use std::sync::Arc;

use super::super::WorthQueryManagedRelationalObservationReleaseReceipt;
use worth_query_admission::facade::resource_admission::WorthQueryExecutionCapacityReservationScope;
use worth_runtime_bridge::facade::{
    BridgeExecutionBasisFinalizationReceipt, BridgeExecutionBasisSignalTerminal,
};

use super::super::{
    WorthQueryManagedProviderWorkEvidence, WorthQueryManagedRunCleanupDisposition,
    WorthQueryManagedRunCounters, WorthQueryYieldCleanupCheckpointInspection,
    WorthQueryYieldRecoveryResourceEvidence, WorthQueryYieldTransitionCounters,
};
use crate::domain_computation::{
    WorthQueryDirectExecutionAttemptReleaseReceipt, WorthQueryProviderCheckpointReleaseEvidence,
};

pub(super) struct WorthQueryCompletedDirectYieldCleanup {
    pub(super) logical_run_identity: Arc<str>,
    pub(super) attempt_identity: Arc<str>,
    pub(super) disposition: WorthQueryManagedRunCleanupDisposition,
    pub(super) checkpoint_release: Option<WorthQueryProviderCheckpointReleaseEvidence>,
    pub(super) recovery_evidence: Option<WorthQueryYieldRecoveryResourceEvidence>,
    pub(super) bridge: BridgeExecutionBasisFinalizationReceipt,
    pub(super) relational: WorthQueryManagedRelationalObservationReleaseReceipt,
    pub(super) attempt: WorthQueryDirectExecutionAttemptReleaseReceipt,
    pub(super) run_counters: WorthQueryManagedRunCounters,
    pub(super) provider_work: WorthQueryManagedProviderWorkEvidence,
    pub(super) yield_counters: WorthQueryYieldTransitionCounters,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryDirectYieldCleanupReceipt {
    inspection: WorthQueryDirectYieldCleanupInspection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryDirectYieldCleanupInspection {
    logical_run_identity: Arc<str>,
    attempt_identity: Arc<str>,
    disposition: WorthQueryManagedRunCleanupDisposition,
    checkpoint: Option<WorthQueryYieldCleanupCheckpointInspection>,
    bridge_reservation_released: bool,
    bridge_signal_terminal: BridgeExecutionBasisSignalTerminal,
    bridge_signal_transition_performed: bool,
    relational_basis_released: bool,
    provider_session_identity: Arc<str>,
    resource_plan_identity: Arc<str>,
    capacity_scope: WorthQueryExecutionCapacityReservationScope,
    released_reservation_count: usize,
    run_counters: WorthQueryManagedRunCounters,
    provider_work: WorthQueryManagedProviderWorkEvidence,
    yield_counters: WorthQueryYieldTransitionCounters,
}

impl WorthQueryDirectYieldCleanupReceipt {
    pub(super) fn from_completed(completed: WorthQueryCompletedDirectYieldCleanup) -> Self {
        let checkpoint_release = completed.checkpoint_release.as_ref().or_else(|| {
            completed
                .recovery_evidence
                .as_ref()
                .and_then(recovery_checkpoint_release)
        });
        let capacity = completed.attempt.capacity();
        let inspection = WorthQueryDirectYieldCleanupInspection {
            logical_run_identity: completed.logical_run_identity,
            attempt_identity: completed.attempt_identity,
            disposition: completed.disposition,
            checkpoint: checkpoint_release.map(WorthQueryYieldCleanupCheckpointInspection::capture),
            bridge_reservation_released: completed.bridge.reservation_released(),
            bridge_signal_terminal: completed.bridge.signal_terminal(),
            bridge_signal_transition_performed: completed.bridge.signal_transition_performed(),
            relational_basis_released: completed.relational.released(),
            provider_session_identity: Arc::from(completed.attempt.provider_session_identity()),
            resource_plan_identity: Arc::from(capacity.resource_plan_identity()),
            capacity_scope: capacity.scope(),
            released_reservation_count: capacity.released_reservation_count(),
            run_counters: completed.run_counters,
            provider_work: completed.provider_work,
            yield_counters: completed.yield_counters,
        };
        Self { inspection }
    }

    pub const fn inspection(&self) -> &WorthQueryDirectYieldCleanupInspection {
        &self.inspection
    }
}

fn recovery_checkpoint_release(
    evidence: &WorthQueryYieldRecoveryResourceEvidence,
) -> Option<&WorthQueryProviderCheckpointReleaseEvidence> {
    evidence.checkpoint_release().or_else(|| {
        evidence
            .provider_checkpoint_failure()
            .and_then(|failure| failure.checkpoint_release())
    })
}

impl WorthQueryDirectYieldCleanupInspection {
    pub fn logical_run_identity(&self) -> &str {
        &self.logical_run_identity
    }
    pub fn yielded_attempt_identity(&self) -> &str {
        &self.attempt_identity
    }
    pub const fn disposition(&self) -> WorthQueryManagedRunCleanupDisposition {
        self.disposition
    }
    pub const fn checkpoint(&self) -> Option<&WorthQueryYieldCleanupCheckpointInspection> {
        self.checkpoint.as_ref()
    }
    pub fn provider_session_identity(&self) -> &str {
        &self.provider_session_identity
    }
    pub fn resource_plan_identity(&self) -> &str {
        &self.resource_plan_identity
    }
    pub const fn capacity_scope(&self) -> WorthQueryExecutionCapacityReservationScope {
        self.capacity_scope
    }
    pub const fn released_reservation_count(&self) -> usize {
        self.released_reservation_count
    }
    pub const fn bridge_signal_terminal(&self) -> BridgeExecutionBasisSignalTerminal {
        self.bridge_signal_terminal
    }
    pub const fn bridge_signal_transition_performed(&self) -> bool {
        self.bridge_signal_transition_performed
    }
    pub const fn resources_released(&self) -> bool {
        self.bridge_reservation_released
            && self.relational_basis_released
            && self.released_reservation_count != 0
    }
    pub const fn provider_work(&self) -> &WorthQueryManagedProviderWorkEvidence {
        &self.provider_work
    }
    pub const fn run_counters(&self) -> &WorthQueryManagedRunCounters {
        &self.run_counters
    }
    pub const fn yield_counters(&self) -> WorthQueryYieldTransitionCounters {
        self.yield_counters
    }
}
