use serde::{Deserialize, Serialize};

use super::{LineageDecisionKind, LineageDecisionRecord};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub(crate) struct LineageDecisionLog {
    decisions: Vec<LineageDecisionRecord>,
}

impl LineageDecisionLog {
    pub(crate) fn new(mut decisions: Vec<LineageDecisionRecord>) -> Self {
        decisions.sort_by(canonical_decision_cmp);
        Self { decisions }
    }

    pub(crate) fn decisions(&self) -> &[LineageDecisionRecord] {
        &self.decisions
    }
}

fn canonical_decision_cmp(
    left: &LineageDecisionRecord,
    right: &LineageDecisionRecord,
) -> std::cmp::Ordering {
    left.branch_id()
        .cmp(right.branch_id())
        .then_with(|| {
            left.event_id()
                .unwrap_or(u64::MAX)
                .cmp(&right.event_id().unwrap_or(u64::MAX))
        })
        .then_with(|| {
            canonical_decision_kind_rank(left.kind().clone())
                .cmp(&canonical_decision_kind_rank(right.kind().clone()))
        })
        .then_with(|| left.sources().cmp(right.sources()))
        .then_with(|| left.targets().cmp(right.targets()))
}

fn canonical_decision_kind_rank(kind: LineageDecisionKind) -> u8 {
    match kind {
        LineageDecisionKind::CreateAccepted => 0,
        LineageDecisionKind::ReplaceAccepted => 1,
        LineageDecisionKind::RetireAccepted => 2,
    }
}
