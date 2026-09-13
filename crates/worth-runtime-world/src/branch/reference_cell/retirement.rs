use crate::identity::CompositeCommitIdentity;

use super::{ProductBranchHeadProtection, ProductBranchReferenceCell};

/// The exact product-head custody released by one cell tombstone.
///
/// Keeping the released protection in this value lets the registry finish its
/// index transition before any retention destructor runs.
pub(crate) struct ProductBranchReferenceRetirement {
    retired_head: CompositeCommitIdentity,
    released_protection: Option<ProductBranchHeadProtection>,
}

impl ProductBranchReferenceRetirement {
    pub(crate) fn into_parts(
        self,
    ) -> (CompositeCommitIdentity, Option<ProductBranchHeadProtection>) {
        (self.retired_head, self.released_protection)
    }
}

impl ProductBranchReferenceCell {
    /// Stop every future movement through this cell and capture its exact final
    /// commit at the same linearization point. The identity clone is prepared
    /// before product-head custody moves out of the active image.
    pub(crate) fn retire(&self) -> ProductBranchReferenceRetirement {
        let mut current = self.state.write();
        let retired_head = current.snapshot.selected_commit().clone();
        let released_protection = current.protection.take();
        ProductBranchReferenceRetirement {
            retired_head,
            released_protection,
        }
    }
}
