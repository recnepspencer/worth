#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BridgeCausalEvidenceOwner {
    Query,
    RuntimeBridge,
    Relational,
    Signal,
}

impl BridgeCausalEvidenceOwner {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::RuntimeBridge => "runtime_bridge",
            Self::Relational => "relational",
            Self::Signal => "signal",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BridgeCausalEvidenceFamily {
    QueryObservation,
    BridgeBulkPlanning,
    BridgeRoute,
    BridgeConditionalObservation,
    BridgeHistoricalEvaluation,
    BridgeHistoricalEvaluationFailure,
    BridgePreviewExecution,
    BridgePreviewDiscard,
    BridgePreviewPromotion,
    BridgeSourceMaterialization,
    BridgeSourceFailure,
    BridgeStructuralRemap,
    BridgeStructuralBranchComparison,
    BridgeStreamReplay,
    BridgeStreamCheckpoint,
    BridgeContinuity,
    BridgeMerge,
    BridgeWritebackAdmission,
    BridgeWritebackMapperEnvelope,
    BridgeWritebackMappedFamilyInput,
    BridgeWritebackMapper,
    BridgeWritebackExecution,
    BridgeWritebackReplay,
    RelationalAuthority,
    SignalInvalidation,
    SignalEvaluation,
    SignalForensicAvailability,
    SignalReplayCursor,
    SignalLineage,
    SignalProvenance,
}

impl BridgeCausalEvidenceFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::QueryObservation => "query_observation",
            Self::BridgeBulkPlanning => "bridge_bulk_planning",
            Self::BridgeRoute => "bridge_route",
            Self::BridgeConditionalObservation => "bridge_conditional_observation",
            Self::BridgeHistoricalEvaluation => "bridge_historical_evaluation",
            Self::BridgeHistoricalEvaluationFailure => "bridge_historical_evaluation_failure",
            Self::BridgePreviewExecution => "bridge_preview_execution",
            Self::BridgePreviewDiscard => "bridge_preview_discard",
            Self::BridgePreviewPromotion => "bridge_preview_promotion",
            Self::BridgeSourceMaterialization => "bridge_source_materialization",
            Self::BridgeSourceFailure => "bridge_source_failure",
            Self::BridgeStructuralRemap => "bridge_structural_remap",
            Self::BridgeStructuralBranchComparison => "bridge_structural_branch_comparison",
            Self::BridgeStreamReplay => "bridge_stream_replay",
            Self::BridgeStreamCheckpoint => "bridge_stream_checkpoint",
            Self::BridgeContinuity => "bridge_continuity",
            Self::BridgeMerge => "bridge_merge",
            Self::BridgeWritebackAdmission => "bridge_writeback_admission",
            Self::BridgeWritebackMapperEnvelope => "bridge_writeback_mapper_envelope",
            Self::BridgeWritebackMappedFamilyInput => "bridge_writeback_mapped_family_input",
            Self::BridgeWritebackMapper => "bridge_writeback_mapper",
            Self::BridgeWritebackExecution => "bridge_writeback_execution",
            Self::BridgeWritebackReplay => "bridge_writeback_replay",
            Self::RelationalAuthority => "relational_authority",
            Self::SignalInvalidation => "signal_invalidation",
            Self::SignalEvaluation => "signal_evaluation",
            Self::SignalForensicAvailability => "signal_forensic_availability",
            Self::SignalReplayCursor => "signal_replay_cursor",
            Self::SignalLineage => "signal_lineage",
            Self::SignalProvenance => "signal_provenance",
        }
    }

    pub fn expected_owner(&self) -> BridgeCausalEvidenceOwner {
        match self {
            Self::QueryObservation => BridgeCausalEvidenceOwner::Query,
            Self::BridgeBulkPlanning
            | Self::BridgeRoute
            | Self::BridgeConditionalObservation
            | Self::BridgeHistoricalEvaluation
            | Self::BridgeHistoricalEvaluationFailure
            | Self::BridgePreviewExecution
            | Self::BridgePreviewDiscard
            | Self::BridgePreviewPromotion
            | Self::BridgeSourceMaterialization
            | Self::BridgeSourceFailure
            | Self::BridgeStructuralRemap
            | Self::BridgeStructuralBranchComparison
            | Self::BridgeStreamReplay
            | Self::BridgeStreamCheckpoint
            | Self::BridgeContinuity
            | Self::BridgeMerge
            | Self::BridgeWritebackAdmission
            | Self::BridgeWritebackMapperEnvelope
            | Self::BridgeWritebackMappedFamilyInput
            | Self::BridgeWritebackMapper
            | Self::BridgeWritebackExecution
            | Self::BridgeWritebackReplay => BridgeCausalEvidenceOwner::RuntimeBridge,
            Self::RelationalAuthority => BridgeCausalEvidenceOwner::Relational,
            Self::SignalInvalidation
            | Self::SignalEvaluation
            | Self::SignalForensicAvailability
            | Self::SignalReplayCursor
            | Self::SignalLineage
            | Self::SignalProvenance => BridgeCausalEvidenceOwner::Signal,
        }
    }
}
