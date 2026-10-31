use crate::identity::CompositeCommitIdentity;

/// Reachability authority and its owner-reserved navigation. Links are derived
/// from installed edges; none grants a protection or keeps a payload alive.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(in crate::history) struct HistoryReachabilityRecord {
    pub(super) descendant_dependencies: usize,
    pub(super) forming_children: usize,
    pub(super) direct_protections: usize,
    pub(super) product_head_protections: usize,
    pub(super) installed_parent: Option<CompositeCommitIdentity>,
    pub(super) first_child: Option<CompositeCommitIdentity>,
    pub(super) previous_sibling: Option<CompositeCommitIdentity>,
    pub(super) next_sibling: Option<CompositeCommitIdentity>,
    pub(super) dirty: bool,
    pub(super) dirty_previous: Option<CompositeCommitIdentity>,
    pub(super) dirty_next: Option<CompositeCommitIdentity>,
}

impl HistoryReachabilityRecord {
    pub(in crate::history::catalog) const fn descendant_dependencies(&self) -> usize {
        self.descendant_dependencies
    }
    pub(in crate::history::catalog) const fn has_forming_child(&self) -> bool {
        self.forming_children > 0
    }
    pub(in crate::history::catalog) const fn direct_protections(&self) -> usize {
        self.direct_protections
    }
    pub(in crate::history::catalog) const fn held_by_product_head(&self) -> bool {
        self.product_head_protections > 0
    }
    pub(in crate::history::catalog) fn sole_installed_child(
        &self,
    ) -> Option<&CompositeCommitIdentity> {
        (self.descendant_dependencies == 1 && self.forming_children == 0)
            .then_some(self.first_child.as_ref())
            .flatten()
    }
}
