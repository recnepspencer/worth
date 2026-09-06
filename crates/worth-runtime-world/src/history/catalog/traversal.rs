use super::CompositeRuntimeWorldCommit;
use std::sync::Arc;

/// Bounded ancestry observation. Its live protection retains the starting
/// commit and its ancestor chain, including their exact component pins.
#[derive(Debug)]
pub struct CompositeHistoryTraversal {
    pub(super) commits: Vec<Arc<CompositeRuntimeWorldCommit>>,
    pub(super) next_parent: Option<crate::identity::CompositeCommitIdentity>,
    pub(super) _catalog: super::CompositeHistoryCatalog,
    pub(super) _protection: super::CompositeHistoryProtectionObligation,
}
impl CompositeHistoryTraversal {
    pub fn commits(&self) -> impl ExactSizeIterator<Item = &CompositeRuntimeWorldCommit> {
        self.commits.iter().map(Arc::as_ref)
    }
    pub fn next_parent(&self) -> Option<&crate::identity::CompositeCommitIdentity> {
        self.next_parent.as_ref()
    }
    pub fn is_complete(&self) -> bool {
        self.next_parent.is_none()
    }
    pub fn visited_count(&self) -> usize {
        self.commits.len()
    }
}
