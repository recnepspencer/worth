use std::sync::Arc;

use super::super::super::WorthQueryManagedRelationalObservationReleaseReceipt;
use worth_query_admission::facade::resource_admission::WorthQueryExecutionCapacityReservationScope;
use worth_runtime_bridge::facade::BridgeExecutionBasisFinalizationReceipt;

use crate::domain_computation::{
    WorthQueryManagedProviderWorkEvidence, WorthQueryManagedRunCleanupDisposition,
    WorthQueryManagedRunCounters, WorthQueryManagedRunTerminalKind,
    WorthQueryWorkflowArtifactRegistryEvidence, WorthQueryWorkflowExecutionAttemptReleaseReceipt,
};

pub(super) struct WorthQueryCompletedWorkflowRunCleanup {
    pub(super) logical_run_identity: Arc<str>,
    pub(super) identity: Arc<str>,
    pub(super) terminal: WorthQueryManagedRunTerminalKind,
    pub(super) disposition: WorthQueryManagedRunCleanupDisposition,
    pub(super) bridge: BridgeExecutionBasisFinalizationReceipt,
    pub(super) relational: WorthQueryManagedRelationalObservationReleaseReceipt,
    pub(super) attempt: WorthQueryWorkflowExecutionAttemptReleaseReceipt,
    pub(super) artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    pub(super) counters: WorthQueryManagedRunCounters,
    pub(super) provider_work: WorthQueryManagedProviderWorkEvidence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryWorkflowRunCleanupReceipt {
    inspection: WorthQueryWorkflowRunCleanupInspection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryWorkflowRunCleanupInspection {
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
    artifact_evidence: WorthQueryWorkflowArtifactRegistryEvidence,
    counters: WorthQueryManagedRunCounters,
    provider_work: WorthQueryManagedProviderWorkEvidence,
}

impl WorthQueryWorkflowRunCleanupReceipt {
    pub(super) fn from_completed(completed: WorthQueryCompletedWorkflowRunCleanup) -> Self {
        let capacity = completed.attempt.capacity();
        let inspection = WorthQueryWorkflowRunCleanupInspection {
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
            artifact_evidence: completed.artifact_evidence,
            counters: completed.counters,
            provider_work: completed.provider_work,
        };
        Self { inspection }
    }

    pub const fn inspection(&self) -> &WorthQueryWorkflowRunCleanupInspection {
        &self.inspection
    }
}

impl WorthQueryWorkflowRunCleanupInspection {
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

    pub fn resources_released(&self) -> bool {
        self.bridge_reservation_released
            && self.relational_basis_released
            && self.released_reservation_count != 0
            && self.artifact_evidence.retained_artifact_count() == 0
            && self.artifact_evidence.retained_bytes() == 0
            && self.artifact_evidence.provider_release_pending_count() == 0
            && self.provider_work.provider_retained_bytes() == 0
    }

    pub const fn artifact_evidence(&self) -> WorthQueryWorkflowArtifactRegistryEvidence {
        self.artifact_evidence
    }

    pub const fn provider_work(&self) -> &WorthQueryManagedProviderWorkEvidence {
        &self.provider_work
    }

    pub const fn counters(&self) -> &WorthQueryManagedRunCounters {
        &self.counters
    }
}
