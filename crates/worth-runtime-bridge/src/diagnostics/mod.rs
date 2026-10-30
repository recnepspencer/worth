pub(crate) mod conditional_history;
pub use conditional_history::{
    BridgeConditionalHistoryAvailability, BridgeConditionalHistoryDetail,
};
mod bulk;
pub(crate) mod causal_envelope;
mod continuity;
mod explanation;
mod facade;
mod failure_source;
mod failure_taxonomy;
mod handle;
mod handle_history;
mod history;
mod merge;
mod policy;
mod records;
mod replay;
mod sink;
mod source;
mod speculation;
mod state;
mod stream;
mod structural;
mod writeback;

pub use bulk::BridgeBulkPlanExplanation;
pub use causal_envelope::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEnvelopeCounters, BridgeCausalEnvelopeDenial,
    BridgeCausalEnvelopeDenialKind, BridgeCausalEnvelopeIdentity, BridgeCausalEnvelopeReceipt,
    BridgeCausalEvidenceBinding, BridgeCausalEvidenceBindingClass, BridgeCausalEvidenceFamily,
    BridgeCausalEvidenceOwner, BridgeCausalEvidenceReference,
    BridgeCausalEvidenceReferenceIdentity, BridgeCausalExplanationEnvelope,
    BridgeCausalInspectionAdmissionSummary, BridgeCausalInspectionAdmissionSummaryKind,
    BridgeConditionalCausalEnvelopeRequest, BridgeQueryObservationReportingInputDenial,
    BridgeQueryObservationReportingRequest, BridgeQueryObservationReportingStop,
};
pub use continuity::{
    BridgeCanonicalContinuityRecord, BridgeContinuityExplanation, BridgeContinuityReplaySummary,
    BridgeDeliveredContinuityResult, BRIDGE_CANONICAL_CONTINUITY_RECORD_SCHEMA_V1,
};
pub use explanation::{BridgeRouteExplanation, BridgeRouteExplanationEntry};
pub use facade::BridgeDiagnosticsFacade;
pub use failure_taxonomy::{
    BridgeFailureEvidenceAttachment, BridgeFailureEvidenceAttachmentSet,
    BridgeFailureLocalizationRequest, BridgeLocalizedTemporalAsyncFailure,
    BridgeTemporalAsyncFailureBundleComparison, BridgeTemporalAsyncFailureClass,
    BridgeTemporalAsyncFailureCounters, BridgeTemporalAsyncFailureLocalizationMatrix,
    BridgeTemporalAsyncFailureLocalizationRejection,
    BridgeTemporalAsyncFailureLocalizationRejectionKind, BridgeTemporalAsyncFailureLocalizationRow,
    BridgeTemporalAsyncFailureSubcode, BridgeTemporalAsyncOfflineDiagnosisBundleDraft,
    BridgeTemporalAsyncOfflineDiagnosisBundleRejection,
    BridgeTemporalAsyncOfflineDiagnosisBundleRejectionKind,
    BridgeTemporalAsyncOfflineDiagnosisBundleSealed,
};
pub use handle::BridgeDiagnosticsHandle;
pub use history::{
    BridgeCanonicalHistoricalEvaluationRecord, BridgeHistoricalEvaluationCounters,
    BridgeHistoricalEvaluationDecisionLog, BridgeHistoricalEvaluationExplanation,
    BridgeHistoricalEvaluationFailureClass, BridgeHistoricalEvaluationFailureIdentity,
    BridgeHistoricalEvaluationFailureRecord, BridgeHistoricalEvaluationRecord,
    BridgeHistoricalEvaluationRecordIdentity, BridgeHistoricalEvaluationReplaySummary,
    BridgeHistoricalMaterializationPath, BRIDGE_CANONICAL_HISTORICAL_EVALUATION_RECORD_SCHEMA_V1,
};
pub use merge::{
    BridgeCanonicalMergeRecord, BridgeMergeExplanation, BridgeMergeRecord,
    BridgeMergeRecordIdentity, BridgeMergeReplaySummary, BRIDGE_CANONICAL_MERGE_RECORD_SCHEMA_V1,
};
pub use policy::{
    BridgePolicyExplanation, BridgePolicyExplanationRow, BridgePolicyRejectionExplanation,
};
pub use records::{
    BridgeContractDiagnosticsRecord, BridgeFailureClass, BridgeFailureRecord,
    BridgeLoweringDiagnosticsRecord, BridgeRouteRecord, BridgeRouteRecordEntityIdentity,
    BridgeRouteRecordEntry, BridgeRouteRecordMatch, BridgeRouteSourceRecord,
    BridgeRoutingDiagnosticsRecord,
};
pub use replay::{
    BridgeCanonicalRouteRecord, BridgeReplayRecord, BridgeReplaySummary,
    BRIDGE_CANONICAL_ROUTE_RECORD_SCHEMA_V3,
};
pub use source::{BridgeSourceFailureExplanation, BridgeSourceMaterializationExplanation};
pub use speculation::{
    BridgePreviewDiscardExplanation, BridgePreviewExecutionExplanation,
    BridgePreviewPromotionExplanation, BridgePreviewReplayExplanation,
};
pub use stream::{
    BridgeStreamCheckpointExplanation, BridgeStreamReplayExplanation, BridgeStreamResumeSummary,
};
pub use structural::{
    BridgeCanonicalStructuralBranchComparisonRecord, BridgeCanonicalStructuralRemapRecord,
    BridgeStructuralBranchComparisonExplanation, BridgeStructuralBranchComparisonRecord,
    BridgeStructuralBranchComparisonReplaySummary, BridgeStructuralCounters,
    BridgeStructuralRemapExplanation, BridgeStructuralRemapRecord,
    BridgeStructuralRemapReplaySummary,
    BRIDGE_CANONICAL_STRUCTURAL_BRANCH_COMPARISON_RECORD_SCHEMA_V1,
    BRIDGE_CANONICAL_STRUCTURAL_REMAP_RECORD_SCHEMA_V1,
};
pub use writeback::{
    BridgeMappedWritebackFamilyInputExplanation, BridgeWritebackAdmissionExplanation,
    BridgeWritebackCandidateExplanation, BridgeWritebackExecutionExplanation,
    BridgeWritebackLoopPreventionExplanation, BridgeWritebackMapperEnvelopeExplanation,
    BridgeWritebackMapperExplanation, BridgeWritebackOutcomeExplanation,
    BridgeWritebackReplayExplanation, BridgeWritebackReplayRecordExplanation,
    BridgeWritebackStrategyCoherenceExplanation,
};

pub(crate) use failure_source::BridgeFailureSource;
pub(crate) use sink::DiagnosticSink;
pub(crate) use structural::{
    validate_structural_replay_contract, validate_structural_replay_outcome,
};
