use super::selection_issuance::{IssuedSelection, SelectionIssuedPayload};
use super::{
    AccessPlanSelectionDenied, SelectedDegradedExactScan, SelectedLsmCompaction, SelectedLsmLookup, SelectedLsmReplayRecovery,
    SelectedLsmRunPublication,
};

#[derive(Debug, PartialEq, Eq)]
pub struct AccessPlanSelectionOutcome {
    issued: IssuedSelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPlanSelectionView<'a> {
    LsmLookup(&'a SelectedLsmLookup),
    LsmRunPublication(&'a SelectedLsmRunPublication),
    LsmReplayRecovery(&'a SelectedLsmReplayRecovery),
    LsmCompaction(&'a SelectedLsmCompaction),
    Degraded(&'a SelectedDegradedExactScan),
    Denied(&'a AccessPlanSelectionDenied),
}

impl AccessPlanSelectionOutcome {
    pub(super) const fn from_issued(issued: IssuedSelection) -> Self {
        Self { issued }
    }

    pub fn view(&self) -> AccessPlanSelectionView<'_> {
        match self.issued.payload() {
            SelectionIssuedPayload::LsmLookup(plan) => AccessPlanSelectionView::LsmLookup(plan),
            SelectionIssuedPayload::LsmRunPublication(plan) => {
                AccessPlanSelectionView::LsmRunPublication(plan)
            }
            SelectionIssuedPayload::LsmReplayRecovery(plan) => {
                AccessPlanSelectionView::LsmReplayRecovery(plan)
            }
            SelectionIssuedPayload::LsmCompaction(plan) => {
                AccessPlanSelectionView::LsmCompaction(plan)
            }
            SelectionIssuedPayload::Degraded(plan) => AccessPlanSelectionView::Degraded(plan),
            SelectionIssuedPayload::Denied(denial) => AccessPlanSelectionView::Denied(denial),
        }
    }

    pub fn case_id(&self) -> super::decision::AccessPlanSelectionCaseId {
        use super::decision::AccessPlanSelectionCaseId as CaseId;

        match self.view() {
            AccessPlanSelectionView::LsmLookup(_) => CaseId::LsmLookup,
            AccessPlanSelectionView::LsmRunPublication(_) => CaseId::LsmRunPublication,
            AccessPlanSelectionView::LsmReplayRecovery(_) => CaseId::LsmReplayRecovery,
            AccessPlanSelectionView::LsmCompaction(_) => CaseId::LsmCompaction,
            AccessPlanSelectionView::Degraded(_) => CaseId::DegradedExactScan,
            AccessPlanSelectionView::Denied(AccessPlanSelectionDenied::NoEligibleAlternative) => {
                CaseId::NoEligibleAlternative
            }
            AccessPlanSelectionView::Denied(AccessPlanSelectionDenied::CostDenied(_)) => {
                CaseId::CostDenied
            }
            AccessPlanSelectionView::Denied(AccessPlanSelectionDenied::BudgetDenied(_)) => {
                CaseId::BudgetDenied
            }
        }
    }

    #[cfg(test)]
    pub fn unwrap_err(self) -> AccessPlanSelectionDenied {
        match self.issued.into_payload() {
            SelectionIssuedPayload::Denied(denial) => denial,
            SelectionIssuedPayload::LsmLookup(_)
            | SelectionIssuedPayload::LsmRunPublication(_)
            | SelectionIssuedPayload::LsmReplayRecovery(_)
            | SelectionIssuedPayload::LsmCompaction(_)
            | SelectionIssuedPayload::Degraded(_) => {
                panic!("selection unexpectedly succeeded")
            }
        }
    }

    pub fn into_lsm_lookup(self) -> Result<SelectedLsmLookup, Self> {
        match self.issued.payload() {
            SelectionIssuedPayload::LsmLookup(_) => {
                let SelectionIssuedPayload::LsmLookup(plan) = self.issued.into_payload() else {
                    unreachable!()
                };
                Ok(plan)
            }
            _ => Err(self),
        }
    }

    pub fn into_lsm_run_publication(self) -> Result<SelectedLsmRunPublication, Self> {
        match self.issued.payload() {
            SelectionIssuedPayload::LsmRunPublication(_) => {
                let SelectionIssuedPayload::LsmRunPublication(plan) = self.issued.into_payload()
                else {
                    unreachable!()
                };
                Ok(plan)
            }
            _ => Err(self),
        }
    }

    pub fn into_lsm_replay_recovery(self) -> Result<SelectedLsmReplayRecovery, Self> {
        match self.issued.payload() {
            SelectionIssuedPayload::LsmReplayRecovery(_) => {
                let SelectionIssuedPayload::LsmReplayRecovery(plan) = self.issued.into_payload()
                else {
                    unreachable!()
                };
                Ok(plan)
            }
            _ => Err(self),
        }
    }

    pub fn into_lsm_compaction(self) -> Result<SelectedLsmCompaction, Self> {
        match self.issued.payload() {
            SelectionIssuedPayload::LsmCompaction(_) => {
                let SelectionIssuedPayload::LsmCompaction(plan) = self.issued.into_payload() else {
                    unreachable!()
                };
                Ok(plan)
            }
            _ => Err(self),
        }
    }

    pub fn into_degraded(self) -> Result<SelectedDegradedExactScan, Self> {
        match self.issued.payload() {
            SelectionIssuedPayload::Degraded(_) => {
                let SelectionIssuedPayload::Degraded(plan) = self.issued.into_payload() else {
                    unreachable!()
                };
                Ok(plan)
            }
            _ => Err(self),
        }
    }
}

pub fn access_plan_selection_cases(
) -> impl Iterator<Item = super::decision::AccessPlanSelectionCaseId> {
    super::decision::AccessPlanSelectionCaseId::ALL.into_iter()
}
