//! Diagnostic custody snapshots use the reclaimer's actual physical-root predicate.
use crate::branch::RelationalBranchRoot;
use std::sync::Arc;

pub(crate) struct RelationalRetiredRootStorageSnapshot {
    pub(crate) root: Arc<RelationalBranchRoot>,
    pub(crate) reclaimable: bool,
}

impl super::RelationalBranchRetentionOwner {
    pub(crate) fn storage_inventory(&self) -> Vec<RelationalRetiredRootStorageSnapshot> {
        self.inner
            .retired_roots
            .iter()
            .map(|entry| {
                // Inspect BEFORE adding this diagnostic's Arc. No obligation is
                // issued, released or inferred from the observation.
                let reclaimable =
                    entry.retired && entry.reservations == 0 && Arc::strong_count(&entry.root) == 1;
                RelationalRetiredRootStorageSnapshot {
                    root: Arc::clone(&entry.root),
                    reclaimable,
                }
            })
            .collect()
    }
}
