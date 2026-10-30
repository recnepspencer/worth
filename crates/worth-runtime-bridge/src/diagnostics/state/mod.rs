use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use crate::input::envelope::TruthCommitIdentity;
use crate::routing::{
    BridgeCanonicalBulkPlanRecord, BridgeInvalidationIdentity, BridgeRouteIdentity,
    BridgeWorkloadIdentity,
};
use crate::source::{SourceFailureRecord, SourceMaterializationRecord};
use crate::stream::{CanonicalStreamReplayRecord, ConsumerCheckpointToken};

use super::continuity::BridgeCanonicalContinuityRecord;
use super::history::{
    BridgeCanonicalHistoricalEvaluationRecord, BridgeHistoricalEvaluationDecisionLogIdentity,
    BridgeHistoricalEvaluationFailureIdentity, BridgeHistoricalEvaluationFailureRecord,
    BridgeHistoricalEvaluationRecordIdentity,
};
use super::merge::{BridgeCanonicalMergeRecord, BridgeMergeRecordIdentity};
use super::records::{BridgeFailureRecord, BridgeRouteRecord};
use super::structural::{
    BridgeCanonicalStructuralBranchComparisonRecord, BridgeCanonicalStructuralRemapRecord,
    BridgeStructuralBranchComparisonRecordIdentity, BridgeStructuralRemapRecordIdentity,
};
use crate::speculation::{
    BridgePreviewDiscardRecord, BridgePreviewDiscardRecordIdentity, BridgePreviewExecutionRecord,
    BridgePreviewPromotionRecord, BridgePreviewPromotionRecordIdentity,
    BridgePreviewSessionIdentity, PreviewExecutionRecordIdentity,
};
use crate::writeback::{
    BridgeMappedWritebackFamilyInput, BridgeMappedWritebackFamilyInputIdentity,
    BridgeWritebackExecutionRecord, BridgeWritebackExecutionRecordIdentity,
    BridgeWritebackFamilyAdmissionRecord, BridgeWritebackFamilyAdmissionRecordIdentity,
    BridgeWritebackMapperEnvelope, BridgeWritebackMapperEnvelopeIdentity,
    BridgeWritebackMapperRecord, BridgeWritebackMapperRecordIdentity, BridgeWritebackReplayRecord,
    BridgeWritebackReplayRecordIdentity,
};

mod config;
mod evict;
mod query;
mod record;
mod speculation;
mod writeback;

pub(crate) use config::BridgeDiagnosticsConfig;

#[derive(Debug, Default)]
pub(crate) struct BridgeDiagnosticsState {
    pub(super) conditional_history: super::conditional_history::ConditionalExecutionHistory,
    route_records: VecDeque<Arc<BridgeRouteRecord>>,
    bulk_records: VecDeque<Arc<BridgeCanonicalBulkPlanRecord>>,
    continuity_records: VecDeque<Arc<BridgeCanonicalContinuityRecord>>,
    merge_records: VecDeque<Arc<BridgeCanonicalMergeRecord>>,
    historical_records: VecDeque<Arc<BridgeCanonicalHistoricalEvaluationRecord>>,
    historical_failures: VecDeque<Arc<BridgeHistoricalEvaluationFailureRecord>>,
    source_materialization_records: VecDeque<Arc<SourceMaterializationRecord>>,
    source_failure_records: VecDeque<Arc<SourceFailureRecord>>,
    structural_remap_records: VecDeque<Arc<BridgeCanonicalStructuralRemapRecord>>,
    structural_branch_comparison_records:
        VecDeque<Arc<BridgeCanonicalStructuralBranchComparisonRecord>>,
    preview_execution_records: VecDeque<Arc<BridgePreviewExecutionRecord>>,
    preview_discard_records: VecDeque<Arc<BridgePreviewDiscardRecord>>,
    preview_promotion_records: VecDeque<Arc<BridgePreviewPromotionRecord>>,
    writeback_admission_records: VecDeque<Arc<BridgeWritebackFamilyAdmissionRecord>>,
    writeback_mapper_envelopes: VecDeque<Arc<BridgeWritebackMapperEnvelope>>,
    writeback_mapped_family_inputs: VecDeque<Arc<BridgeMappedWritebackFamilyInput>>,
    writeback_execution_records: VecDeque<Arc<BridgeWritebackExecutionRecord>>,
    writeback_mapper_records: VecDeque<Arc<BridgeWritebackMapperRecord>>,
    writeback_replay_records: VecDeque<Arc<BridgeWritebackReplayRecord>>,
    stream_checkpoints: VecDeque<Arc<ConsumerCheckpointToken>>,
    stream_replay_records: VecDeque<Arc<CanonicalStreamReplayRecord>>,
    failure_records: VecDeque<Arc<BridgeFailureRecord>>,
    latest_route_by_route_identity: BTreeMap<BridgeRouteIdentity, Arc<BridgeRouteRecord>>,
    latest_bulk_by_workload_identity: BTreeMap<String, Arc<BridgeCanonicalBulkPlanRecord>>,
    latest_continuity_by_route_identity:
        BTreeMap<BridgeRouteIdentity, Arc<BridgeCanonicalContinuityRecord>>,
    latest_merge_by_record_identity: BTreeMap<String, Arc<BridgeCanonicalMergeRecord>>,
    latest_historical_by_record_identity:
        BTreeMap<String, Arc<BridgeCanonicalHistoricalEvaluationRecord>>,
    latest_historical_by_decision_log_identity:
        BTreeMap<String, Arc<BridgeCanonicalHistoricalEvaluationRecord>>,
    latest_historical_failure_by_declaration_identity:
        BTreeMap<String, Arc<BridgeHistoricalEvaluationFailureRecord>>,
    latest_historical_failure_by_failure_identity:
        BTreeMap<String, Arc<BridgeHistoricalEvaluationFailureRecord>>,
    latest_source_materialization_by_record_identity:
        BTreeMap<String, Arc<SourceMaterializationRecord>>,
    latest_source_failure_by_declaration_identity: BTreeMap<String, Arc<SourceFailureRecord>>,
    latest_source_failure_by_failure_identity: BTreeMap<String, Arc<SourceFailureRecord>>,
    latest_structural_remap_by_record_identity:
        BTreeMap<String, Arc<BridgeCanonicalStructuralRemapRecord>>,
    latest_structural_branch_comparison_by_record_identity:
        BTreeMap<String, Arc<BridgeCanonicalStructuralBranchComparisonRecord>>,
    latest_preview_execution_by_record_identity:
        BTreeMap<String, Arc<BridgePreviewExecutionRecord>>,
    latest_preview_execution_by_session_identity:
        BTreeMap<String, Arc<BridgePreviewExecutionRecord>>,
    latest_preview_discard_by_record_identity: BTreeMap<String, Arc<BridgePreviewDiscardRecord>>,
    latest_preview_discard_by_session_identity: BTreeMap<String, Arc<BridgePreviewDiscardRecord>>,
    latest_preview_promotion_by_record_identity:
        BTreeMap<String, Arc<BridgePreviewPromotionRecord>>,
    latest_preview_promotion_by_session_identity:
        BTreeMap<String, Arc<BridgePreviewPromotionRecord>>,
    latest_writeback_admission_by_record_identity:
        BTreeMap<String, Arc<BridgeWritebackFamilyAdmissionRecord>>,
    latest_writeback_admission_by_contract_digest:
        BTreeMap<String, Arc<BridgeWritebackFamilyAdmissionRecord>>,
    latest_writeback_mapper_envelope_by_identity:
        BTreeMap<String, Arc<BridgeWritebackMapperEnvelope>>,
    latest_writeback_mapper_envelope_by_digest:
        BTreeMap<String, Arc<BridgeWritebackMapperEnvelope>>,
    latest_writeback_mapped_input_by_identity:
        BTreeMap<String, Arc<BridgeMappedWritebackFamilyInput>>,
    latest_writeback_mapped_input_by_digest:
        BTreeMap<String, Arc<BridgeMappedWritebackFamilyInput>>,
    latest_writeback_execution_by_record_identity:
        BTreeMap<String, Arc<BridgeWritebackExecutionRecord>>,
    latest_writeback_execution_by_candidate_digest:
        BTreeMap<String, Arc<BridgeWritebackExecutionRecord>>,
    latest_writeback_mapper_by_record_identity: BTreeMap<String, Arc<BridgeWritebackMapperRecord>>,
    latest_writeback_mapper_by_candidate_digest: BTreeMap<String, Arc<BridgeWritebackMapperRecord>>,
    latest_writeback_replay_by_record_identity: BTreeMap<String, Arc<BridgeWritebackReplayRecord>>,
    reserved_preview_session_identities: BTreeSet<BridgePreviewSessionIdentity>,
    latest_stream_checkpoint_by_identity: BTreeMap<String, Arc<ConsumerCheckpointToken>>,
    latest_stream_replay_by_identity: BTreeMap<String, Arc<CanonicalStreamReplayRecord>>,
    latest_stream_replay_by_checkpoint_identity: BTreeMap<String, Arc<CanonicalStreamReplayRecord>>,
    latest_route_by_invalidation_identity:
        BTreeMap<BridgeInvalidationIdentity, Arc<BridgeRouteRecord>>,
    latest_route_by_source_commit: BTreeMap<TruthCommitIdentity, Arc<BridgeRouteRecord>>,
}
