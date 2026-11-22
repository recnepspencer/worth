use std::sync::Arc;

use super::super::WorthQueryManagedRelationalObservationReleaseReceipt;
use worth_query_admission::facade::resource_admission::WorthQueryExecutionCapacityReservationScope;
use worth_runtime_bridge::facade::BridgeExecutionBasisFinalizationReceipt;

use super::super::{
    WorthQueryManagedProviderWorkEvidence, WorthQueryManagedRunCleanupDisposition,
    WorthQueryManagedRunCounters, WorthQueryManagedRunTerminalKind,
};
use crate::domain_computation::WorthQueryDirectExecutionAttemptReleaseReceipt;

pub(super) struct WorthQueryCompletedDirectRunCleanup {
    pub(super) logical_run_identity: Arc<str>,
    pub(super) identity: Arc<str>,
    pub(super) terminal: WorthQueryManagedRunTerminalKind,
    pub(super) disposition: WorthQueryManagedRunCleanupDisposition,
    pub(super) bridge: BridgeExecutionBasisFinalizationReceipt,
    pub(super) relational: WorthQueryManagedRelationalObservationReleaseReceipt,
    pub(super) attempt: WorthQueryDirectExecutionAttemptReleaseReceipt,
    pub(super) counters: WorthQueryManagedRunCounters,
    pub(super) provider_work: WorthQueryManagedProviderWorkEvidence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryDirectRunCleanupReceipt {
    inspection: WorthQueryDirectRunCleanupInspection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryDirectRunCleanupInspection {
    logical_run_identity: Arc<str>,
    identity: Arc<str>,
    terminal: WorthQueryManagedRunTerminalKind,
    disposition: WorthQueryManagedRunCleanupDisposition,
    bridge_reservation_released: bool,
    relational_basis_released: bool,
    provider_session_identity: Arc<str>,
    resource_plan_identity: Arc<str>,
    capacity_scope: WorthQueryExecutionCapacityReservationScope,
    released_reservation_count: usize,
    counters: WorthQueryManagedRunCounters,
    provider_work: WorthQueryManagedProviderWorkEvidence,
}

impl WorthQueryDirectRunCleanupReceipt {
    pub(super) fn from_completed(completed: WorthQueryCompletedDirectRunCleanup) -> Self {
        let capacity = completed.attempt.capacity();
        let inspection = WorthQueryDirectRunCleanupInspection {
            logical_run_identity: completed.logical_run_identity,
            identity: completed.identity,
            terminal: completed.terminal,
            disposition: completed.disposition,
            bridge_reservation_released: completed.bridge.reservation_released(),
            relational_basis_released: completed.relational.released(),
            provider_session_identity: Arc::from(completed.attempt.provider_session_identity()),
            resource_plan_identity: Arc::from(capacity.resource_plan_identity()),
            capacity_scope: capacity.scope(),
            released_reservation_count: capacity.released_reservation_count(),
            counters: completed.counters,
            provider_work: completed.provider_work,
        };
        Self { inspection }
    }

    pub const fn inspection(&self) -> &WorthQueryDirectRunCleanupInspection {
        &self.inspection
    }
}

impl WorthQueryDirectRunCleanupInspection {
    pub fn run_identity(&self) -> &str {
        &self.identity
    }

    pub fn logical_run_identity(&self) -> &str {
        &self.logical_run_identity
    }

    pub const fn terminal(&self) -> WorthQueryManagedRunTerminalKind {
        self.terminal
    }

    pub const fn disposition(&self) -> WorthQueryManagedRunCleanupDisposition {
        self.disposition
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

    pub const fn resources_released(&self) -> bool {
        self.bridge_reservation_released
            && self.relational_basis_released
            && self.released_reservation_count != 0
    }

    pub const fn provider_work(&self) -> &WorthQueryManagedProviderWorkEvidence {
        &self.provider_work
    }

    pub const fn counters(&self) -> &WorthQueryManagedRunCounters {
        &self.counters
    }
}
