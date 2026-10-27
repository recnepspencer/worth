use worth_foundational::facade::AspectKey;

use super::{operation_rank, PublishedAuthoritativePatch};

impl PublishedAuthoritativePatch {
    /// Canonical operations are ordered by grammar rank, then aspect key.
    /// Search each existing rank without scanning unrelated aspect operations.
    pub(crate) fn changes_aspect(&self, aspect: &AspectKey) -> bool {
        (0..=2).any(|rank| {
            self.operations
                .binary_search_by(|operation| {
                    operation_rank(operation)
                        .cmp(&rank)
                        .then_with(|| operation.aspect_key().cmp(aspect))
                })
                .is_ok()
        })
    }
}
