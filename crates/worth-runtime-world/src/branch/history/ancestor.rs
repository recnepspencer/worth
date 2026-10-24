//! An inherited source occurrence bound to an authentic target observation.
use crate::history::{
    CompositeAncestorSelectionDenial, CompositeRuntimeWorldCommit, RetainedCompositeAncestor,
};

use super::super::{ProductBranchObservation, RuntimeWorldBranchAdmissionDenial};

/// Retains historical meaning on one exact target ancestry. Preparing effects
/// still requires fresh admission and comparison of the target head.
#[derive(Debug)]
pub struct ProductBranchAncestorSelection {
    target: ProductBranchObservation,
    ancestor: RetainedCompositeAncestor,
}

impl ProductBranchAncestorSelection {
    pub(crate) fn owner_issued(
        target: ProductBranchObservation,
        ancestor: RetainedCompositeAncestor,
    ) -> Self {
        assert_eq!(target.selected_commit(), ancestor.target());
        Self { target, ancestor }
    }

    pub fn target(&self) -> &ProductBranchObservation {
        &self.target
    }

    pub fn source(&self) -> &CompositeRuntimeWorldCommit {
        self.ancestor.source()
    }

    /// Native parent links followed during this exact ancestry selection.
    pub fn parent_links_followed(&self) -> usize {
        self.ancestor.parent_links_followed()
    }
}

/// Identifies whether exact ancestor selection stopped at current branch
/// admission or at retained-history lookup. The nested cause preserves the
/// owner's distinction between stale selection, unavailable history and work
/// that reached its requested traversal bound.
#[derive(Debug)]
pub enum ProductBranchAncestorSelectionDenial {
    Branch(RuntimeWorldBranchAdmissionDenial),
    History(CompositeAncestorSelectionDenial),
}
