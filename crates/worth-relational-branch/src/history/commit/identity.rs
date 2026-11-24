use serde::{Deserialize, Serialize};

use crate::history::data::{BranchId, CommitId};
use crate::identity::data::VersionId;

/// Immutable identity of one committed history fact.
///
/// Branch ownership is deliberately retained as authoring provenance only. It
/// never selects a current branch head; that decision belongs to a branch
/// reference cell.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RelationalCommitIdentity {
    commit_id: CommitId,
    version_id: VersionId,
    authoring_branch: BranchId,
}

impl RelationalCommitIdentity {
    pub const fn commit_id(&self) -> CommitId {
        self.commit_id
    }

    pub const fn version_id(&self) -> VersionId {
        self.version_id
    }

    pub fn authoring_branch(&self) -> &BranchId {
        &self.authoring_branch
    }
}

/// Operations used by the Relational owners.
pub trait RelationalCommitIdentityOwnerPort: Sized {
    fn new(commit_id: CommitId, version_id: VersionId, authoring_branch: BranchId) -> Self;
}

impl RelationalCommitIdentityOwnerPort for RelationalCommitIdentity {
    fn new(commit_id: CommitId, version_id: VersionId, authoring_branch: BranchId) -> Self {
        Self {
            commit_id,
            version_id,
            authoring_branch,
        }
    }
}
