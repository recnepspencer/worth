//! Exactly one source of proposed meaning enters the ordinary judging round.
use crate::authority::commit::semantic_plan::retained_entity_payload::PreparedRetainedEntityPayload;
use crate::contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState;

pub enum RelationalProposalSource {
    Transaction {
        lowered: Option<(SelectedRelationalBranchState, MergedCommitPlan)>,
        strategy: Option<StrategyProposalDecoration>,
    },
    RetainedEntitySelection(PreparedRetainedEntityPayload),
    OwnedGraphSelection(crate::mvcc::owned_graph_selection::prepared::PreparedOwnedGraphSelection),
}

impl RelationalProposalSource {
    pub fn into_strategy(self) -> Option<StrategyProposalDecoration> {
        match self {
            Self::Transaction { strategy, .. } => strategy,
            Self::RetainedEntitySelection(_) | Self::OwnedGraphSelection(_) => None,
        }
    }
}

pub struct StrategyProposalDecoration {
    pub artifacts: crate::contracts::commit_strategies::data::artifacts::artifact_bundle::StrategyCommitArtifactBundle,
    pub bulk_mutation_batch:
        Option<crate::contracts::transactions::data::primitives::ProvenanceCompleteBulkMutationBatch>,
}
