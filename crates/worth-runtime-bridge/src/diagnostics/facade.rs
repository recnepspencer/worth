use std::sync::{Arc, RwLock};

use crate::error::{BridgeDeliveryError, BridgeReplayError};
use crate::policy::{BridgeDiagnosticsTier, BridgeRuntimePolicy};
use crate::routing::BridgeCanonicalBulkPlanRecord;
use crate::speculation::{
    BridgePreviewDiscardRecord, BridgePreviewExecutionRecord, BridgePreviewPromotionRecord,
    BridgePreviewSessionIdentity,
};
use crate::stream::{CanonicalStreamReplayRecord, ConsumerCheckpointToken};
use crate::writeback::{
    BridgeMappedWritebackFamilyInput, BridgeWritebackExecutionRecord,
    BridgeWritebackFamilyAdmissionRecord, BridgeWritebackMapperEnvelope,
    BridgeWritebackMapperRecord, BridgeWritebackReplayRecord,
};

use super::bulk::BridgeBulkPlanExplanation;
use super::continuity::{BridgeCanonicalContinuityRecord, BridgeContinuityExplanation};
use super::failure_source::BridgeFailureSource;
use super::handle::BridgeDiagnosticsHandle;
use super::history::{
    BridgeCanonicalHistoricalEvaluationRecord, BridgeHistoricalEvaluationExplanation,
    BridgeHistoricalEvaluationFailureRecord,
};
use super::merge::{BridgeCanonicalMergeRecord, BridgeMergeExplanation};
use super::records::{BridgeFailureClass, BridgeFailureRecord, BridgeRouteRecord};
use super::replay::{BridgeCanonicalRouteRecord, BridgeReplayRecord};
use super::sink::DiagnosticSink;
use super::speculation::{
    BridgePreviewDiscardExplanation, BridgePreviewExecutionExplanation,
    BridgePreviewPromotionExplanation, BridgePreviewReplayExplanation,
};
use super::state::{BridgeDiagnosticsConfig, BridgeDiagnosticsState};
use super::stream::{BridgeStreamCheckpointExplanation, BridgeStreamReplayExplanation};
use super::structural::{
    BridgeCanonicalStructuralBranchComparisonRecord, BridgeCanonicalStructuralRemapRecord,
};
use super::writeback::{
    BridgeMappedWritebackFamilyInputExplanation, BridgeWritebackAdmissionExplanation,
    BridgeWritebackCandidateExplanation, BridgeWritebackExecutionExplanation,
    BridgeWritebackLoopPreventionExplanation, BridgeWritebackMapperEnvelopeExplanation,
    BridgeWritebackMapperExplanation, BridgeWritebackOutcomeExplanation,
    BridgeWritebackReplayExplanation, BridgeWritebackReplayRecordExplanation,
    BridgeWritebackStrategyCoherenceExplanation,
};
use super::BridgeRouteExplanation;

mod conditional_history;
mod explain;
mod policy;
mod query;
mod record;
mod sink;
mod speculation;
mod writeback;

#[derive(Debug, Clone)]
pub struct BridgeDiagnosticsFacade {
    config: Arc<BridgeDiagnosticsConfig>,
    state: Arc<RwLock<BridgeDiagnosticsState>>,
}
