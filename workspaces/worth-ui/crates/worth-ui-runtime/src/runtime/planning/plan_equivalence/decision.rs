use super::{
    WorthUiExecutablePlanDecisionKind, WorthUiExecutablePlanEquivalenceDenial,
    WorthUiPlanEquivalenceSummary,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiExecutablePlanDecision {
    ExactSemanticNoOp(WorthUiPlanEquivalenceSummary),
    BoundedChangedRegions(WorthUiPlanEquivalenceSummary),
    RebuildRequired(WorthUiPlanEquivalenceSummary),
    Denied(WorthUiExecutablePlanEquivalenceDenial),
}

impl WorthUiExecutablePlanDecision {
    pub fn kind(self) -> WorthUiExecutablePlanDecisionKind {
        match self {
            Self::ExactSemanticNoOp(_) => WorthUiExecutablePlanDecisionKind::ExactSemanticNoOp,
            Self::BoundedChangedRegions(_) => {
                WorthUiExecutablePlanDecisionKind::BoundedChangedRegions
            }
            Self::RebuildRequired(_) => WorthUiExecutablePlanDecisionKind::RebuildRequired,
            Self::Denied(_) => WorthUiExecutablePlanDecisionKind::Denied,
        }
    }

    pub fn summary(self) -> Option<WorthUiPlanEquivalenceSummary> {
        match self {
            Self::ExactSemanticNoOp(summary)
            | Self::BoundedChangedRegions(summary)
            | Self::RebuildRequired(summary) => Some(summary),
            Self::Denied(_) => None,
        }
    }

    pub fn denial(self) -> Option<WorthUiExecutablePlanEquivalenceDenial> {
        match self {
            Self::Denied(denial) => Some(denial),
            Self::ExactSemanticNoOp(_)
            | Self::BoundedChangedRegions(_)
            | Self::RebuildRequired(_) => None,
        }
    }
}
