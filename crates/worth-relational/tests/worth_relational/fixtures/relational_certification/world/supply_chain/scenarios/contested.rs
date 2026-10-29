pub(crate) use super::{BaselineName, BranchCreationIntent, SupplyChainBaseline};
pub(crate) use crate::fast::relational_certification::world::supply_chain::scale::SupplyChainScale;
pub(crate) use crate::fast::relational_certification::world::supply_chain::semantic_key::BranchLabel;

pub(crate) fn build(scale: SupplyChainScale) -> SupplyChainBaseline {
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
