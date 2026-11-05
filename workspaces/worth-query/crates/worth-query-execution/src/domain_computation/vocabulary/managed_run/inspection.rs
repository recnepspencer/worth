//! Moved whole from `domain_computation::managed_run::direct_terminal::inspection`: the items below and every impl they own.
use crate::domain_computation::vocabulary::managed_run::counters::WorthQueryManagedRunCounters;
use crate::domain_computation::vocabulary::managed_run::evidence::WorthQueryManagedProviderWorkEvidence;
use crate::domain_computation::vocabulary::managed_run::relational_observation::WorthQueryManagedRelationalObservationReleaseReceipt;
use crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunCleanupDisposition;
use crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunTerminalKind;
use crate::domain_computation::vocabulary::provider_session::direct_attempt::WorthQueryDirectExecutionAttemptReleaseReceipt;
use std::sync::Arc;
use worth_query_admission::facade::resource_admission::WorthQueryExecutionCapacityReservationScope;
use worth_runtime_bridge::facade::BridgeExecutionBasisFinalizationReceipt;

pub(in crate::domain_computation) struct WorthQueryCompletedDirectRunCleanup {
    pub(in crate::domain_computation) logical_run_identity: Arc<str>,
    pub(in crate::domain_computation) identity: Arc<str>,
    pub(in crate::domain_computation) terminal: WorthQueryManagedRunTerminalKind,
    pub(in crate::domain_computation) disposition: WorthQueryManagedRunCleanupDisposition,
    pub(in crate::domain_computation) bridge: BridgeExecutionBasisFinalizationReceipt,
    pub(in crate::domain_computation) relational:
        WorthQueryManagedRelationalObservationReleaseReceipt,
    pub(in crate::domain_computation) attempt: WorthQueryDirectExecutionAttemptReleaseReceipt,
    pub(in crate::domain_computation) counters: WorthQueryManagedRunCounters,
    pub(in crate::domain_computation) provider_work: WorthQueryManagedProviderWorkEvidence,
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
    pub(in crate::domain_computation) fn from_completed(
        completed: WorthQueryCompletedDirectRunCleanup,
    ) -> Self {
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
