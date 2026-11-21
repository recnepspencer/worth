//! Internal crate-boundary port for finalizing a branch owner cleanup receipt
//! when a published World continuation has already released every owner custody.

use worth_runtime_world::facade::CompositeCommitIdentity;

use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt;

/// Internal crate-boundary port for issuing a completed cleanup receipt when a
/// published World continuation has already released every owner custody.
pub trait ProductBranchOwnerCleanupPort {
    fn continued_original_for_published(
        commit: &CompositeCommitIdentity,
    ) -> WorthQueryProductBranchOwnerCleanupReceipt;
}

impl ProductBranchOwnerCleanupPort for WorthQueryProductBranchOwnerCleanupReceipt {
    fn continued_original_for_published(
        _commit: &CompositeCommitIdentity,
    ) -> WorthQueryProductBranchOwnerCleanupReceipt {
        Self::continued_original()
    }
}
