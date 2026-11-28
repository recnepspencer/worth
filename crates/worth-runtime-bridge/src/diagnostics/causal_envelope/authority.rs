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
    BridgeContinuity,
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
            Self::BridgeContinuity => "bridge_continuity",
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
            | Self::BridgeContinuity => BridgeCausalEvidenceOwner::RuntimeBridge,
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
