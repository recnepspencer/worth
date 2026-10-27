use crate::identity::CompositeCommitIdentity;
#[cfg(test)]
use crate::identity::RuntimeWorldOwnerIdentity;

use super::catalog::CompositeHistoryCatalogDenial;

/// A bounded batch the catalog's own tests reclaim directly. Every owner path
/// releases history through the walk, `release_retired_branch_history`, so
/// no door outside these tests removes a commit without walking to it.
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompositeHistoryReclamationRequest {
    owner: RuntimeWorldOwnerIdentity,
    candidate_commits: Vec<CompositeCommitIdentity>,
    maximum_reclaims: usize,
}

#[cfg(test)]
impl CompositeHistoryReclamationRequest {
    pub fn new(
        owner: RuntimeWorldOwnerIdentity,
        candidate_commits: Vec<CompositeCommitIdentity>,
        maximum_reclaims: usize,
    ) -> Self {
        Self {
            owner,
            candidate_commits,
            maximum_reclaims,
        }
    }

    pub const fn owner(&self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }

    pub fn candidate_commits(&self) -> &[CompositeCommitIdentity] {
        &self.candidate_commits
    }

    pub const fn maximum_reclaims(&self) -> usize {
        self.maximum_reclaims
    }
}

/// Reclamation is an observation of a bounded maintenance batch, not a
/// promise that every requested candidate can be removed.
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HistoryReclamationOutcome {
    maximum_reclaims: usize,
    examined: usize,
    skipped_protected: usize,
    skipped_with_descendant_dependencies: usize,
    reclaimed: Vec<CompositeCommitIdentity>,
}

#[cfg(test)]
impl HistoryReclamationOutcome {
    pub(crate) fn new(maximum_reclaims: usize) -> Self {
        Self {
            maximum_reclaims,
            examined: 0,
            skipped_protected: 0,
            skipped_with_descendant_dependencies: 0,
            reclaimed: Vec::new(),
        }
    }

    pub(crate) fn examined_one(&mut self) {
        self.examined += 1;
    }

    pub(crate) fn record_skipped_protected(&mut self) {
        self.skipped_protected += 1;
    }

    pub(crate) fn record_skipped_with_descendant_dependencies(&mut self) {
        self.skipped_with_descendant_dependencies += 1;
    }

    pub(crate) fn reclaimed_one(&mut self, identity: CompositeCommitIdentity) {
        debug_assert!(self.reclaimed.len() < self.maximum_reclaims);
        self.reclaimed.push(identity);
    }

    pub const fn examined(&self) -> usize {
        self.examined
    }

    pub const fn skipped_protected(&self) -> usize {
        self.skipped_protected
    }

    pub const fn skipped_with_descendant_dependencies(&self) -> usize {
        self.skipped_with_descendant_dependencies
    }

    pub fn reclaimed_commits(&self) -> &[CompositeCommitIdentity] {
        &self.reclaimed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryReclamationDenial {
    OwnerUnavailable(crate::lifecycle::RuntimeWorldOwnerUnavailable),
    Catalog(CompositeHistoryCatalogDenial),
    // A batch's candidates are refused whole; only the tests' batches name them.
    #[cfg(test)]
    ForeignCandidate {
        expected: RuntimeWorldOwnerIdentity,
        actual: RuntimeWorldOwnerIdentity,
    },
    #[cfg(test)]
    DuplicateCandidate(CompositeCommitIdentity),
    #[cfg(test)]
    UnknownCandidate(CompositeCommitIdentity),
}

/// What releasing a closed branch's history left behind. The walk removes
/// every commit only that branch needed, from its retired head toward the
/// root, and stops at the first commit something else still needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetiredBranchHistoryRelease {
    reclaimed: Vec<CompositeCommitIdentity>,
    held: Option<CompositeCommitIdentity>,
}

impl RetiredBranchHistoryRelease {
    pub(crate) fn new(
        reclaimed: Vec<CompositeCommitIdentity>,
        held: Option<CompositeCommitIdentity>,
    ) -> Self {
        Self { reclaimed, held }
    }

    /// Commits removed by this walk, newest first.
    pub fn reclaimed_commits(&self) -> &[CompositeCommitIdentity] {
        &self.reclaimed
    }

    /// The commit a held read, receipt, forming child or unconsumed delivery
    /// still needs. Nothing else owns the history from it down, so the closed
    /// branch resumes its release here once that holder lets go. `None` means
    /// the release is complete: what remains is a live branch's head or a
    /// live descendant's ancestry.
    pub fn held_commit(&self) -> Option<&CompositeCommitIdentity> {
        self.held.as_ref()
    }
}
