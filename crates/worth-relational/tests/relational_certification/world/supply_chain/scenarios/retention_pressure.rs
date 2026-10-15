use super::{BaselineName, RetentionObligation, RetentionObligationKind, SupplyChainBaseline};
use crate::relational_certification::world::supply_chain::scale::SupplyChainScale;
use crate::relational_certification::world::supply_chain::semantic_key::BranchLabel;

pub(super) fn build(scale: SupplyChainScale) -> SupplyChainBaseline {
    let mut baseline = super::contested::build(scale);
    baseline.name = BaselineName::RetentionPressure;
    baseline.retention_obligations = vec![
        RetentionObligation {
            target: BranchLabel::Maintenance,
            ancestor_path: vec![BranchLabel::Operating],
            kind: RetentionObligationKind::Snapshot,
        },
        RetentionObligation {
            target: BranchLabel::Maintenance,
            ancestor_path: vec![BranchLabel::Operating, BranchLabel::Storm],
            kind: RetentionObligationKind::Observation,
        },
        RetentionObligation {
            target: BranchLabel::Maintenance,
            ancestor_path: vec![BranchLabel::Operating, BranchLabel::Maintenance],
            kind: RetentionObligationKind::Transaction,
        },
        RetentionObligation {
            target: BranchLabel::Maintenance,
            ancestor_path: vec![BranchLabel::Operating, BranchLabel::Customs],
            kind: RetentionObligationKind::Candidate,
        },
        RetentionObligation {
            target: BranchLabel::Maintenance,
            ancestor_path: vec![BranchLabel::Operating, BranchLabel::Rewire],
            kind: RetentionObligationKind::ExternalBasis,
        },
    ];
    baseline
        .validate_retention_obligations()
        .expect("canonical retention paths and targets must be declared");
    baseline
}
