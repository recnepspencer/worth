use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PreparationFailureClass {
    PlanningProofInsufficient,
    ReductionIdentityConflict,
    SerialStrategySelected,
    PublicationIsolationViolation,
}

impl PreparationFailureClass {
    pub(crate) const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::PlanningProofInsufficient => "planning_proof_insufficient",
            Self::ReductionIdentityConflict => "reduction_identity_conflict",
            Self::SerialStrategySelected => "serial_strategy_selected",
            Self::PublicationIsolationViolation => "publication_isolation_violation",
        }
    }
}
