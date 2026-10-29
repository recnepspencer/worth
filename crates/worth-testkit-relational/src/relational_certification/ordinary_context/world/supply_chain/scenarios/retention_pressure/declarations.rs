pub use {crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName, crate::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligation, crate::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationKind, crate::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel};

pub fn build(scale: SupplyChainScale) -> SupplyChainBaseline {
    let mut baseline = crate::relational_certification::ordinary_context::world::supply_chain::scenarios::contested::build(scale);
    baseline.name = crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::RetentionPressure;
    baseline.retention_obligations = vec![
        RetentionObligation {
            target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance,
            ancestor_path: vec![crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating],
            kind: crate::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationKind::Snapshot,
        },
        RetentionObligation {
            target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance,
            ancestor_path: vec![crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm],
            kind: crate::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationKind::Observation,
        },
        RetentionObligation {
            target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance,
            ancestor_path: vec![crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance],
            kind: crate::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationKind::Transaction,
        },
        RetentionObligation {
            target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance,
            ancestor_path: vec![crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs],
            kind: crate::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationKind::Candidate,
        },
        RetentionObligation {
            target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance,
            ancestor_path: vec![crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire],
            kind: crate::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationKind::ExternalBasis,
        },
    ];
    baseline
        .validate_retention_obligations()
        .expect("canonical retention paths and targets must be declared");
    baseline
}