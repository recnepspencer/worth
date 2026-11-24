//! Exactly one source of proposed meaning enters the ordinary judging round.
use crate::authority::commit::semantic_plan::retained_entity_payload::PreparedRetainedEntityPayload;
use crate::branch::root_selection::SelectedRelationalBranchState;
use crate::mvcc::validation::validated_proposal::StrategyProposalDecoration;
use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;

pub(super) enum RelationalProposalSource {
    Transaction {
        lowered: Option<(SelectedRelationalBranchState, MergedCommitPlan)>,
        strategy: Option<StrategyProposalDecoration>,
    },
    RetainedEntitySelection(PreparedRetainedEntityPayload),
    OwnedGraphSelection(crate::mvcc::owned_graph_selection::prepared::PreparedOwnedGraphSelection),
}

impl RelationalProposalSource {
    pub(super) fn into_strategy(self) -> Option<StrategyProposalDecoration> {
        match self {
            Self::Transaction { strategy, .. } => strategy,
            Self::RetainedEntitySelection(_) | Self::OwnedGraphSelection(_) => None,
        }
    }
}
