use std::sync::{Arc, RwLock};

use crate::routing::{BridgeCanonicalBulkPlanRecord, BridgeWorkloadIdentity};

use super::continuity::BridgeCanonicalContinuityRecord;
use super::records::{BridgeFailureRecord, BridgeRouteRecord};
use super::state::{BridgeDiagnosticsConfig, BridgeDiagnosticsState};

#[derive(Debug, Clone)]
pub struct BridgeDiagnosticsHandle {
    pub(super) config: Arc<BridgeDiagnosticsConfig>,
    pub(super) state: Arc<RwLock<BridgeDiagnosticsState>>,
}

impl BridgeDiagnosticsHandle {
    pub fn tier(&self) -> crate::policy::BridgeDiagnosticsTier {
        self.config.tier
    }

    pub fn records_enabled(&self) -> bool {
        self.config.records_enabled
    }

    pub fn replay_enabled(&self) -> bool {
        self.config.replay_enabled
    }

    pub fn route_record_limit(&self) -> usize {
        self.config.route_record_limit
    }

    pub fn failure_record_limit(&self) -> usize {
        self.config.failure_record_limit
    }

    pub fn route_records(&self) -> Vec<BridgeRouteRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .route_records()
    }

    pub fn failure_records(&self) -> Vec<BridgeFailureRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .failure_records()
    }

    pub fn bulk_records(&self) -> Vec<BridgeCanonicalBulkPlanRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .bulk_records()
    }

    pub fn continuity_records(&self) -> Vec<BridgeCanonicalContinuityRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .continuity_records()
    }

    pub fn bulk_record_for_workload_identity(
        &self,
        workload_identity: &BridgeWorkloadIdentity,
    ) -> Option<BridgeCanonicalBulkPlanRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .bulk_record_for_workload_identity(workload_identity)
    }
}
