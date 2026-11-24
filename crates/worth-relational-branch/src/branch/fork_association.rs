//! The exact fork result keeps its original acknowledged publication witness.
use super::RelationalForkOutcome;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::transition::RelationalCommittedTransitionEvidence;
use worth_proof::DurablyCommitted;

#[derive(Debug)]
pub struct RelationalAssociatedFork {
    outcome: RelationalForkOutcome,
    basis: AdmittedRelationalBranchBasis,
    evidence: DurablyCommitted<RelationalCommittedTransitionEvidence>,
}

impl RelationalAssociatedFork {
    pub fn outcome(&self) -> &RelationalForkOutcome {
        &self.outcome
    }

    pub fn basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.basis
    }

    pub fn into_parts(
        self,
    ) -> (
        RelationalForkOutcome,
        AdmittedRelationalBranchBasis,
        DurablyCommitted<RelationalCommittedTransitionEvidence>,
    ) {
        (self.outcome, self.basis, self.evidence)
    }
}

/// Operations used by the Relational owners.
pub trait RelationalAssociatedForkOwnerPort: Sized {
    fn from_parts(
        outcome: RelationalForkOutcome,
        basis: AdmittedRelationalBranchBasis,
        evidence: DurablyCommitted<RelationalCommittedTransitionEvidence>,
    ) -> Self;
}

impl RelationalAssociatedForkOwnerPort for RelationalAssociatedFork {
    fn from_parts(
        outcome: RelationalForkOutcome,
        basis: AdmittedRelationalBranchBasis,
        evidence: DurablyCommitted<RelationalCommittedTransitionEvidence>,
    ) -> Self {
        Self {
            outcome,
            basis,
            evidence,
        }
    }
}
