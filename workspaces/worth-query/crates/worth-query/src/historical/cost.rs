#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum HistoricalPathCostPosture {
    EvidenceIdentityDerivationDenied,
    HistoricalRetainedFastPath,
    HistoricalReplayBounded,
    HistoricalReconstructionExpensive,
    HistoricalPathDeniedByBudget,
    HistoricalPathDeniedByCompatibility,
    HistoricalPathDeniedByUnsupportedPath,
    HistoricalPathSubstitutionDenied,
}

impl HistoricalPathCostPosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HistoricalRetainedFastPath => "historical_retained_fast_path",
            Self::HistoricalReplayBounded => "historical_replay_bounded",
            Self::HistoricalReconstructionExpensive => "historical_reconstruction_expensive",
            Self::HistoricalPathDeniedByBudget => "historical_path_denied_by_budget",
            Self::HistoricalPathDeniedByCompatibility => "historical_path_denied_by_compatibility",
            Self::HistoricalPathDeniedByUnsupportedPath => {
                "historical_path_denied_by_unsupported_path"
            }
            Self::HistoricalPathSubstitutionDenied => "historical_path_substitution_denied",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RetainedStateReuseEligibility {
    Reusable,
    NotReusable,
}

impl RetainedStateReuseEligibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Reusable => "reusable",
            Self::NotReusable => "not_reusable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayTailReuseEligibility {
    Reusable,
    NotReusable,
}

impl ReplayTailReuseEligibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Reusable => "reusable",
            Self::NotReusable => "not_reusable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PerformancePredictionDriftOutcome {
    WithinBudget,
    StructuralCandidatePredictionDrift,
    HistoricalReplaySpanDrift,
    HistoricalReconstructionScopeDrift,
}

impl PerformancePredictionDriftOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WithinBudget => "within_budget",
            Self::StructuralCandidatePredictionDrift => "structural_candidate_prediction_drift",
            Self::HistoricalReplaySpanDrift => "historical_replay_span_drift",
            Self::HistoricalReconstructionScopeDrift => "historical_reconstruction_scope_drift",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoricalPerformanceStatusMarker {
    Verified,
    Debt,
}

impl HistoricalPerformanceStatusMarker {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Debt => "debt",
        }
    }
}
