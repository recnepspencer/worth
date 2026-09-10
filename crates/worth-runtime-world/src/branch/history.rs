use std::num::NonZeroUsize;
use std::sync::Arc;

use crate::history::{CompositeHistoryTraversal, CompositeRuntimeWorldCommit};
use crate::identity::{
    CompositeCommitIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductBranchReferenceGeneration,
};

/// Owner-issued bounded history for one live product-branch occurrence.
/// Entries retain their exact component bases through the inner World history
/// protection; descriptive identities cannot construct this carrier.
#[derive(Debug)]
pub struct ProductBranchHistoryTraversal {
    branch: ProductBranchIdentity,
    occurrence: ProductBranchIncarnation,
    first_generation: ProductBranchReferenceGeneration,
    traversal: CompositeHistoryTraversal,
    remaining_branch_commits: usize,
}

impl ProductBranchHistoryTraversal {
    pub(crate) fn owner_issued(
        branch: ProductBranchIdentity,
        occurrence: ProductBranchIncarnation,
        first_generation: ProductBranchReferenceGeneration,
        traversal: CompositeHistoryTraversal,
        remaining_branch_commits: usize,
    ) -> Self {
        Self {
            branch,
            occurrence,
            first_generation,
            traversal,
            remaining_branch_commits,
        }
    }

    pub fn commits(&self) -> impl ExactSizeIterator<Item = &CompositeRuntimeWorldCommit> {
        self.traversal.commits()
    }

    pub fn visited_count(&self) -> usize {
        self.traversal.visited_count()
    }

    pub fn branch_identity(&self) -> &ProductBranchIdentity {
        &self.branch
    }

    pub fn lifecycle_incarnation(&self) -> ProductBranchIncarnation {
        self.occurrence
    }

    pub fn is_complete(&self) -> bool {
        self.remaining_branch_commits == 0 || self.traversal.is_complete()
    }

    pub fn next_parent(&self) -> Option<&CompositeCommitIdentity> {
        (!self.is_complete())
            .then(|| self.traversal.next_parent())
            .flatten()
    }

    pub(crate) fn occurrence(&self) -> ProductBranchIncarnation {
        self.occurrence
    }

    pub(crate) fn branch(&self) -> &ProductBranchIdentity {
        &self.branch
    }

    pub(crate) fn continuation_limit(&self, maximum: NonZeroUsize) -> Option<NonZeroUsize> {
        NonZeroUsize::new(maximum.get().min(self.remaining_branch_commits))
    }

    pub(crate) fn selection(
        &self,
        index: usize,
    ) -> Option<(
        Arc<CompositeRuntimeWorldCommit>,
        ProductBranchReferenceGeneration,
    )> {
        let commit = self.traversal.shared_commit(index)?;
        let generation = self.first_generation.retreat(index)?;
        Some((commit, generation))
    }

    pub(crate) fn continued(&self, traversal: CompositeHistoryTraversal) -> Option<Self> {
        let visited = traversal.visited_count();
        let first_generation = self
            .first_generation
            .retreat(self.traversal.visited_count())?;
        Some(Self::owner_issued(
            self.branch.clone(),
            self.occurrence,
            first_generation,
            traversal,
            self.remaining_branch_commits.checked_sub(visited)?,
        ))
    }
}
