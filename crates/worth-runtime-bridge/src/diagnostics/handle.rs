use std::sync::{Arc, RwLock};

use crate::routing::{BridgeCanonicalBulkPlanRecord, BridgeWorkloadIdentity};
use crate::speculation::{
    BridgePreviewDiscardRecord, BridgePreviewDiscardRecordIdentity, BridgePreviewExecutionRecord,
    BridgePreviewPromotionRecord, BridgePreviewPromotionRecordIdentity,
    BridgePreviewSessionIdentity, PreviewExecutionRecordIdentity,
};

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

    pub fn preview_execution_records(&self) -> Vec<BridgePreviewExecutionRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .preview_execution_records()
    }

    pub fn preview_discard_records(&self) -> Vec<BridgePreviewDiscardRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .preview_discard_records()
    }

    pub fn preview_promotion_records(&self) -> Vec<BridgePreviewPromotionRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .preview_promotion_records()
    }

    pub fn last_preview_execution_record(&self) -> Option<BridgePreviewExecutionRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .last_preview_execution_record()
    }

    pub fn last_preview_discard_record(&self) -> Option<BridgePreviewDiscardRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .last_preview_discard_record()
    }

    pub fn last_preview_promotion_record(&self) -> Option<BridgePreviewPromotionRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .last_preview_promotion_record()
    }

    pub fn preview_execution_record_for_identity(
        &self,
        record_identity: &PreviewExecutionRecordIdentity,
    ) -> Option<BridgePreviewExecutionRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .preview_execution_record_for_identity(record_identity)
    }

    pub fn preview_execution_record_for_session_identity(
        &self,
        session_identity: &BridgePreviewSessionIdentity,
    ) -> Option<BridgePreviewExecutionRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .preview_execution_record_for_session_identity(session_identity)
    }

    pub fn preview_discard_record_for_identity(
        &self,
        record_identity: &BridgePreviewDiscardRecordIdentity,
    ) -> Option<BridgePreviewDiscardRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .preview_discard_record_for_identity(record_identity)
    }

    pub fn preview_discard_record_for_session_identity(
        &self,
        session_identity: &BridgePreviewSessionIdentity,
    ) -> Option<BridgePreviewDiscardRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .preview_discard_record_for_session_identity(session_identity)
    }

    pub fn preview_promotion_record_for_identity(
        &self,
        record_identity: &BridgePreviewPromotionRecordIdentity,
    ) -> Option<BridgePreviewPromotionRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .preview_promotion_record_for_identity(record_identity)
    }

    pub fn preview_promotion_record_for_session_identity(
        &self,
        session_identity: &BridgePreviewSessionIdentity,
    ) -> Option<BridgePreviewPromotionRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .preview_promotion_record_for_session_identity(session_identity)
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
