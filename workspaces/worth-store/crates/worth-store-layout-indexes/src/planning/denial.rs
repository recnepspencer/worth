use crate::strategy::registry::{LayoutAdmissionDenial, LayoutAdmissionDenialCase};
use crate::strategy::LayoutStrategyFamily;
use worth_store_budgets::PreExecutionBudgetDenial;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionCandidateRejection {
    RegistryDenied(Box<LayoutAdmissionDenial>),
    OperationUnsupported {
        family: LayoutStrategyFamily,
        shape: crate::observation::AccessShape,
    },
    MissingPlannedCounterEnvelope,
    NotApplicableToExplicitDegradedScan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectionCandidateRejectionCase {
    RegistryDenied(LayoutAdmissionDenialCase),
    OperationUnsupported,
    MissingPlannedCounterEnvelope,
    NotApplicableToExplicitDegradedScan,
}

impl SelectionCandidateRejection {
    pub const fn case(&self) -> SelectionCandidateRejectionCase {
        match self {
            Self::RegistryDenied(denial) => {
                SelectionCandidateRejectionCase::RegistryDenied(denial.case())
            }
            Self::OperationUnsupported { .. } => {
                SelectionCandidateRejectionCase::OperationUnsupported
            }
            Self::MissingPlannedCounterEnvelope => {
                SelectionCandidateRejectionCase::MissingPlannedCounterEnvelope
            }
            Self::NotApplicableToExplicitDegradedScan => {
                SelectionCandidateRejectionCase::NotApplicableToExplicitDegradedScan
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessPlanSelectionDenied {
    NoEligibleAlternative,
    CostDenied(super::AccessPlanCostDenial),
    BudgetDenied(PreExecutionBudgetDenial),
}
