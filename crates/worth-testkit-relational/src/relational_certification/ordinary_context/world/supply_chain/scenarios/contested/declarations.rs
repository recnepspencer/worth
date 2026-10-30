pub use crate::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale;
pub use crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel;
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName,
    crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BranchCreationIntent,
    crate::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline,
};

pub fn build(scale: SupplyChainScale) -> SupplyChainBaseline {
    let mut baseline = crate::relational_certification::ordinary_context::world::supply_chain::scenarios::operating::build(scale);
    baseline.name = crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::ContestedPlanning;
    baseline.branch_intents = vec![
        BranchCreationIntent {
            branch: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            parent: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
        },
        BranchCreationIntent {
            branch: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance,
            parent: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
        },
        BranchCreationIntent {
            branch: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs,
            parent: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
        },
        BranchCreationIntent {
            branch: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire,
            parent: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
        },
    ];
    baseline
        .validate_branch_intents()
        .expect("canonical contested intents must be legal");
    baseline
}
