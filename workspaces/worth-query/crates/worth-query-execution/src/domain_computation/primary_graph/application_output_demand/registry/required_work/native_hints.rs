//! One native prepared attempt per owned node, including across branch cells.

use std::sync::Arc;

use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::CompanionPublicationCompletionObserver;

use super::RequiredWorkMembership;

pub(in crate::domain_computation::primary_graph) struct NativeHint {
    pub(super) observer: CompanionPublicationCompletionObserver,
    pub(super) branch: Arc<NativeBranch>,
    // The selection version at which this publication was seen installed
    // (or aborted) before a head check. Only such a hint is covered.
    pub(super) seen_settled_at: Option<u64>,
    pub(super) next: Option<Box<NativeHint>>,
}

/// The final selected Arc, not the hint node, owns branch text and control
/// block capacity. A stale selected worker can outlive an acknowledged node.
pub(in crate::domain_computation::primary_graph) struct NativeBranch {
    branch: BranchId,
}

impl NativeBranch {
    pub(in crate::domain_computation::primary_graph) fn branch_id(&self) -> &BranchId {
        &self.branch
    }
}

impl RequiredWorkMembership {
    pub(in crate::domain_computation::primary_graph) fn prepared_native_branch(
        branch: BranchId,
    ) -> Arc<NativeBranch> {
        Arc::new(NativeBranch { branch })
    }

    /// Construction happens only after native preflight pays the Box and
    /// retained ticket. Installation only links this already-owned node.
    pub(in crate::domain_computation::primary_graph) fn prepared_native_hint(
        observer: CompanionPublicationCompletionObserver,
        branch: Arc<NativeBranch>,
    ) -> Box<NativeHint> {
        Box::new(NativeHint {
            observer,
            branch,
            seen_settled_at: None,
            next: None,
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

impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
}
