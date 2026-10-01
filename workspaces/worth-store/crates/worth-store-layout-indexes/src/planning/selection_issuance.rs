use super::decision::PlanSelectionDecision;
use super::{
    AccessPlanSelectionDenied, SelectedDegradedExactScan, SelectedLsmCompaction, SelectedLsmLookup, SelectedLsmReplayRecovery,
    SelectedLsmRunPublication,
};
#[derive(Debug, PartialEq, Eq)]
pub(super) enum SelectionIssuedPayload {
    LsmLookup(SelectedLsmLookup),
    LsmRunPublication(SelectedLsmRunPublication),
    LsmReplayRecovery(SelectedLsmReplayRecovery),
    LsmCompaction(SelectedLsmCompaction),
    Degraded(SelectedDegradedExactScan),
    Denied(AccessPlanSelectionDenied),
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct IssuedSelection {
    payload: SelectionIssuedPayload,
}

impl IssuedSelection {
    pub(super) const fn payload(&self) -> &SelectionIssuedPayload {
        &self.payload
    }

    pub(super) fn into_payload(self) -> SelectionIssuedPayload {
        self.payload
    }
}

pub(super) fn issue_selection_outcome(
    decision: PlanSelectionDecision,
) -> super::AccessPlanSelectionOutcome {
    let payload = match decision {
        PlanSelectionDecision::LsmLookup(plan, grant) => {
            SelectionIssuedPayload::LsmLookup(SelectedLsmLookup::from_decision(plan, grant))
        }
        PlanSelectionDecision::LsmRunPublication(plan, grant) => {
            SelectionIssuedPayload::LsmRunPublication(SelectedLsmRunPublication::from_decision(
                plan, grant,
            ))
        }
        PlanSelectionDecision::LsmReplayRecovery(plan, grant) => {
            SelectionIssuedPayload::LsmReplayRecovery(SelectedLsmReplayRecovery::from_decision(
                plan, grant,
            ))
        }
        PlanSelectionDecision::LsmCompaction(plan, grant) => {
            SelectionIssuedPayload::LsmCompaction(SelectedLsmCompaction::from_decision(plan, grant))
        }
        PlanSelectionDecision::Degraded(plan, grant) => {
            SelectionIssuedPayload::Degraded(SelectedDegradedExactScan::from_decision(plan, grant))
        }
        PlanSelectionDecision::Denied(denial) => SelectionIssuedPayload::Denied(denial),
    };
    super::AccessPlanSelectionOutcome::from_issued(IssuedSelection { payload })
}
