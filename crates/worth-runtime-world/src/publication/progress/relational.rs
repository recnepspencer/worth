use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::branch::RelationalForkOutcome;
use worth_relational::facade::history::RelationalCommitIdentity;

use super::{
    RelationalAttemptProgress, RelationalAttemptProgressPosture, RelationalProgressEvidence,
};

impl RelationalAttemptProgress {
    /// The only Relational fork progress. A fork creates a branch and never
    /// carries commit evidence, so it has no settlement obligation.
    pub(crate) fn forked(
        fork: RelationalForkOutcome,
        successor_basis: AdmittedRelationalBranchBasis,
    ) -> Self {
        Self {
            posture: RelationalAttemptProgressPosture::Performed,
            evidence: None,
            fork: Some(fork),
            fork_successor_basis: Some(successor_basis),
        }
    }

    pub(crate) fn settlement_required(
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
    ) -> Self {
        Self {
            posture: RelationalAttemptProgressPosture::SettlementRequired,
            evidence: Some(RelationalProgressEvidence::SettlementRequired {
                commit_identity,
                successor_basis,
            }),
            fork: None,
            fork_successor_basis: None,
        }
    }

    pub(crate) fn requires_settlement(&self) -> bool {
        matches!(
            self.evidence.as_ref(),
            Some(
                RelationalProgressEvidence::Performed(_)
                    | RelationalProgressEvidence::SettlementRequired { .. }
                    | RelationalProgressEvidence::SettlementPending { .. }
            )
        )
    }

    pub(crate) fn is_fork_only(&self) -> bool {
        self.posture == RelationalAttemptProgressPosture::Performed
            && self.evidence.is_none()
            && self.fork.is_some()
            && self.fork_successor_basis.is_some()
    }

    pub(crate) fn settled_receipt(
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
        receipt: worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Self {
        Self {
            posture: RelationalAttemptProgressPosture::Settled,
            evidence: Some(RelationalProgressEvidence::SettledReceipt {
                commit_identity,
                successor_basis,
                receipt,
            }),
            fork: None,
            fork_successor_basis: None,
        }
    }
}
