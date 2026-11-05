//! Moved whole from `domain_computation::primary_graph::application_output_demand::registry::required_work::native_hints`: the items below and every impl they own.
use std::sync::Arc;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::CompanionPublicationCompletionObserver;

pub struct NativeHint {
    pub(in crate::domain_computation::primary_graph) observer: CompanionPublicationCompletionObserver,
    pub(in crate::domain_computation::primary_graph) branch: Arc<NativeBranch>,
    // The selection version at which this publication was seen installed
    // (or aborted) before a head check. Only such a hint is covered.
    pub(in crate::domain_computation::primary_graph) seen_settled_at: Option<u64>,
    pub(in crate::domain_computation::primary_graph) next: Option<Box<NativeHint>>,
}

/// The final selected Arc, not the hint node, owns branch text and control
/// block capacity. A stale selected worker can outlive an acknowledged node.
pub struct NativeBranch {
    branch: BranchId,
}

impl NativeBranch {
    pub(in crate::domain_computation::primary_graph) fn branch_id(&self) -> &BranchId {
        &self.branch
    }
}

impl Drop for NativeHint {
    fn drop(&mut self) {
        // Arbitrarily many distinct branch-cell attempts cannot recurse
        // through Box<NativeHint> on the final membership owner.
        let mut next = self.next.take();
        while let Some(mut node) = next {
            next = node.next.take();
        }
    }
}

impl NativeBranch {
    #[doc(hidden)]
    pub fn branch_id_for_test(&self) -> &BranchId {
        Self::branch_id(self)
    }
}
