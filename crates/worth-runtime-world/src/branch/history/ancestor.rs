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
}

#[derive(Debug)]
pub enum ProductBranchAncestorSelectionDenial {
    Branch(RuntimeWorldBranchAdmissionDenial),
    History(CompositeAncestorSelectionDenial),
}
