#[derive(Clone, Copy)]
pub enum FixtureFamilyMismatch {
    Universe,
    Termination,
    Feasibility,
    Comparison,
    Incumbent,
    Progress,
    Comparator,
    RepeatedState,
}

#[derive(Clone, Copy)]
pub enum FixtureDisposition {
    Continue,
    Converged,
    StableWithoutProof,
    FeasibleIncumbent,
    Oscillating,
    OscillatingSelected,
    DomainClassifiedOscillation,
    RepeatedContinue,
    Stalled,
    IndeterminateComparison,
    IncoherentStable,
    ComparatorFailure,
    ComparatorPanic,
    ProgressFailure,
    ProgressPanic,
    RepeatedStateFailure,
    RepeatedStatePanic,
    FamilyInspectionPanic,
    YieldThenCheckpointUnavailable,
    YieldThenConverged,
    YieldThenRestorePanic,
    YieldThenCheckpointDropPanic,
    YieldThenSuspensionFailure,
    ChunkedConverged(usize),
    StageQueueContractMismatch,
    ParetoReplacement,
    ParetoCollision,
    ParetoPartialReplacement,
    HistoryRetain,
    HistoryClear,
    HistoryInvalidTransition,
    HistoryInvalidDomain,
    FamilyMismatch(FixtureFamilyMismatch),
}

impl FixtureDisposition {
    pub fn mismatches(self, family: FixtureFamilyMismatch) -> bool {
        matches!(self, Self::FamilyMismatch(actual) if same_family(actual, family))
    }

    pub const fn projection_width(self) -> Option<usize> {
        match self {
            Self::ChunkedConverged(width) => Some(width),
            _ => None,
        }
    }
}

pub fn same_family(left: FixtureFamilyMismatch, right: FixtureFamilyMismatch) -> bool {
    std::mem::discriminant(&left) == std::mem::discriminant(&right)
}
