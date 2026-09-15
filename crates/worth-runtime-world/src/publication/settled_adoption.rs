use std::sync::Arc;

use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::history::RelationalCommitIdentity;
use worth_relational::facade::transactions::CommitResult;

/// World-owned evidence that a retained publication already completed and
/// settled its Relational leg. Only the recovery catalog can mint this value.
/// A later product publication may adopt it, but cannot replay the owner work.
#[derive(Debug)]
pub(crate) struct SettledRelationalPublicationAdoption {
    commit_identity: RelationalCommitIdentity,
    successor_basis: AdmittedRelationalBranchBasis,
    result: Arc<CommitResult>,
}

impl SettledRelationalPublicationAdoption {
    pub(crate) fn new(
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
        result: Arc<CommitResult>,
    ) -> Self {
        Self {
            commit_identity,
            successor_basis,
            result,
        }
    }

    pub(crate) fn successor_basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.successor_basis
    }

    pub(crate) fn into_progress(self) -> super::RelationalAttemptProgress {
        super::RelationalAttemptProgress::settled_arc(
            self.commit_identity,
            self.successor_basis,
            self.result,
        )
    }
}
