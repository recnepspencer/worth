//! The exact fork result keeps its original acknowledged publication witness.
use super::publication::ForkInstallationOutcome;
use super::RelationalForkOutcome;
use crate::branch::{AdmittedRelationalBranchBasis, RelationalCommittedTransitionEvidence};
use worth_proof::DurablyCommitted;

#[derive(Debug)]
pub struct RelationalAssociatedFork {
    outcome: RelationalForkOutcome,
    basis: AdmittedRelationalBranchBasis,
    evidence: DurablyCommitted<RelationalCommittedTransitionEvidence>,
}

impl RelationalAssociatedFork {
    pub(super) fn from_installation(installed: ForkInstallationOutcome) -> Self {
        Self {
            outcome: installed.outcome,
            basis: installed
                .basis
                .expect("associated fork requests its owner-admitted basis"),
            evidence: installed
                .evidence
                .expect("associated live fork requires an attached durable medium"),
        }
    }

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
