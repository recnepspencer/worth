//! One native prepared attempt per owned node, including across branch cells.

use std::sync::Arc;

use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::CompanionPublicationCompletionObserver;

use super::RequiredWorkMembership;
use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;

pub(in crate::domain_computation::primary_graph) struct NativeHint {
    pub(super) observer: CompanionPublicationCompletionObserver,
    pub(super) branch: Arc<FundedNativeBranch>,
    // The selection version at which this publication was seen installed
    // (or aborted) before a head check. Only such a hint is covered.
    pub(super) seen_settled_at: Option<u64>,
    pub(super) next: Option<Box<NativeHint>>,
    _capacity: Arc<RetainedInvalidationCapacity>,
}

/// The final selected Arc, not the hint node, owns branch text and control
/// block capacity. A stale selected worker can outlive an acknowledged node.
pub(in crate::domain_computation::primary_graph) struct FundedNativeBranch {
    branch: BranchId,
    _capacity: Arc<RetainedInvalidationCapacity>,
}

impl FundedNativeBranch {
    pub(in crate::domain_computation::primary_graph) fn branch_id(&self) -> &BranchId {
        &self.branch
    }
}

impl RequiredWorkMembership {
    pub(in crate::domain_computation::primary_graph) const fn native_hint_bytes() -> u64 {
        std::mem::size_of::<NativeHint>() as u64
    }

    pub(in crate::domain_computation::primary_graph) fn native_branch_retained_bytes(
        branch_name_bytes: usize,
    ) -> Option<u64> {
        let (layout, _) = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<FundedNativeBranch>())
            .ok()?;
        let bytes = layout
            .pad_to_align()
            .size()
            .checked_add(branch_name_bytes)?;
        u64::try_from(bytes).ok()
    }

    pub(in crate::domain_computation::primary_graph) fn prepared_native_branch(
        branch: BranchId,
        capacity: Arc<RetainedInvalidationCapacity>,
    ) -> Arc<FundedNativeBranch> {
        Arc::new(FundedNativeBranch {
            branch,
            _capacity: capacity,
        })
    }

    /// Construction happens only after native preflight pays the Box and
    /// retained ticket. Installation only links this already-owned node.
    pub(in crate::domain_computation::primary_graph) fn prepared_native_hint(
        observer: CompanionPublicationCompletionObserver,
        branch: Arc<FundedNativeBranch>,
        capacity: Arc<RetainedInvalidationCapacity>,
    ) -> Box<NativeHint> {
        Box::new(NativeHint {
            observer,
            branch,
            seen_settled_at: None,
            next: None,
            _capacity: capacity,
        })
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
