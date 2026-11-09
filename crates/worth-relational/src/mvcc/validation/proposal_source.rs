//! Exactly one source of proposed meaning enters the ordinary judging round.
use super::StrategyProposalDecoration;
use crate::authority::commit::semantic_plan::PreparedRetainedEntityPayload;
use crate::branch::SelectedRelationalBranchState;
use crate::transactions::data::MergedCommitPlan;

pub(super) enum RelationalProposalSource {
    Transaction {
        lowered: Option<(SelectedRelationalBranchState, MergedCommitPlan)>,
        strategy: Option<StrategyProposalDecoration>,
    },
    RetainedEntitySelection(PreparedRetainedEntityPayload),
    OwnedGraphSelection(crate::mvcc::owned_graph_selection::PreparedOwnedGraphSelection),
}

impl RelationalProposalSource {
    pub(super) fn into_strategy(self) -> Option<StrategyProposalDecoration> {
        match self {
            Self::Transaction { strategy, .. } => strategy,
            Self::RetainedEntitySelection(_) | Self::OwnedGraphSelection(_) => None,
        }
    }
}
