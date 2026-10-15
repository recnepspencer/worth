use super::{BaselineName, BranchCreationIntent, SupplyChainBaseline};
use crate::relational_certification::world::supply_chain::scale::SupplyChainScale;
use crate::relational_certification::world::supply_chain::semantic_key::BranchLabel;

pub(super) fn build(scale: SupplyChainScale) -> SupplyChainBaseline {
    let mut baseline = super::operating::build(scale);
    baseline.name = BaselineName::ContestedPlanning;
    baseline.branch_intents = vec![
        BranchCreationIntent {
            branch: BranchLabel::Storm,
            parent: BranchLabel::Operating,
        },
        BranchCreationIntent {
            branch: BranchLabel::Maintenance,
            parent: BranchLabel::Operating,
        },
        BranchCreationIntent {
            branch: BranchLabel::Customs,
            parent: BranchLabel::Operating,
        },
        BranchCreationIntent {
            branch: BranchLabel::Rewire,
            parent: BranchLabel::Operating,
        },
    ];
    baseline
        .validate_branch_intents()
        .expect("canonical contested intents must be legal");
    baseline
}
