use std::sync::Arc;

use crate::storage::substrate::StorageAllocationOverflow;

/// Exact roots and publishing branches that can still select a derived cache.
/// Retired roots contribute only owner-recorded active obligations, not Arc
/// payload counts or a stale entry in the retired-root inventory.
#[derive(Debug, Default)]
pub(crate) struct RetainedIndexRoots {
    pub(crate) live: std::collections::BTreeSet<(
        crate::history::data::BranchId,
        crate::identity::data::VersionId,
        crate::schema::data::SchemaVersionId,
    )>,
    pub(crate) retired: std::collections::BTreeSet<(
        crate::history::data::BranchId,
        crate::identity::data::VersionId,
        crate::schema::data::SchemaVersionId,
    )>,
    pub(crate) historical_versions: std::collections::BTreeSet<(
        crate::identity::data::VersionId,
        crate::schema::data::SchemaVersionId,
    )>,
    pub(crate) latest_branches: std::collections::BTreeSet<crate::history::data::BranchId>,
}

/// Bounded maintenance result for retired immutable branch roots.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RelationalBranchRootReclamationOutcome {
    roots_examined: u64,
    roots_reclaimed: u64,
    roots_still_retained: u64,
    unique_authoritative_bytes_reclaimed: u64,
    accounting_overflow: bool,
}

impl super::RelationalBranchRetentionOwner {
    pub(crate) fn try_reclaim_retired_roots(
        &self,
        maximum_roots: usize,
    ) -> Result<RelationalBranchRootReclamationOutcome, StorageAllocationOverflow> {
        let mut examined = 0_u64;
        let mut reclaimed = 0_u64;
        let mut unique_bytes = 0_u64;
        let pass_limit = maximum_roots.min(self.inner.retired_root_order.len());
        for _ in 0..pass_limit {
            let Some((root_key, generation)) = self.inner.retired_root_order.pop() else {
                break;
            };
            let next_examined = examined.checked_add(1).ok_or(StorageAllocationOverflow)?;
            let reclaimable = {
                let mut roots = self.inner.write_retired_roots();
                let removable = roots.get(&root_key).is_some_and(|retired| {
                    retired.generation == generation
                        && retired.retired
                        && retired.reservations == 0
                        && Arc::strong_count(&retired.root) == 1
                });
                if removable {
                    let root_bytes = roots[&root_key]
                        .root
                        .reclaimable_unique_authoritative_bytes()?;
                    let next_unique_bytes = unique_bytes
                        .checked_add(root_bytes)
                        .ok_or(StorageAllocationOverflow)?;
                    let next_reclaimed =
                        reclaimed.checked_add(1).ok_or(StorageAllocationOverflow)?;
                    let counters = super::owner::lock_maintenance_counters(&self.inner);
                    counters
                        .reclamation_roots_examined
                        .checked_add(next_examined)
                        .ok_or(StorageAllocationOverflow)?;
                    counters
                        .reclamation_roots_reclaimed
                        .checked_add(next_reclaimed)
                        .ok_or(StorageAllocationOverflow)?;
                    counters
                        .reclamation_unique_authoritative_bytes
                        .checked_add(next_unique_bytes)
                        .ok_or(StorageAllocationOverflow)?;
                    drop(counters);
                    roots.remove(&root_key).map(|retired| (retired, root_bytes))
                } else {
                    None
                }
            };
            examined = next_examined;
            if let Some((retired, root_bytes)) = reclaimable {
                let root = retired.root;
                self.inner
                    .retired_root_locators
                    .remove(&root, root_key, generation);
                super::owner::release_retirement_slot(&self.inner);
                self.inner
                    .retired_root_count
                    .fetch_sub(1, std::sync::atomic::Ordering::Release);
                unique_bytes = unique_bytes
                    .checked_add(root_bytes)
                    .ok_or(StorageAllocationOverflow)?;
                reclaimed = reclaimed.checked_add(1).ok_or(StorageAllocationOverflow)?;
                drop(root);
            } else if self
                .inner
                .read_retired_roots()
                .get(&root_key)
                .is_some_and(|entry| entry.generation == generation && entry.retired)
            {
                self.inner.retired_root_order.push((root_key, generation));
            }
        }

        let roots_still_retained = self
            .inner
            .retired_root_count
            .load(std::sync::atomic::Ordering::Acquire) as u64;
        super::cost_counters::try_record_reclamation(
            &mut super::owner::lock_maintenance_counters(&self.inner),
            examined,
            reclaimed,
            unique_bytes,
        )?;
        Ok(RelationalBranchRootReclamationOutcome::new(
            examined,
            reclaimed,
            roots_still_retained,
            unique_bytes,
        ))
    }

    pub(crate) fn reclaim_retired_roots(
        &self,
        maximum_roots: usize,
    ) -> RelationalBranchRootReclamationOutcome {
        match self.try_reclaim_retired_roots(maximum_roots) {
            Ok(outcome) => outcome,
            Err(StorageAllocationOverflow) => RelationalBranchRootReclamationOutcome {
                roots_still_retained: self
                    .inner
                    .retired_root_count
                    .load(std::sync::atomic::Ordering::Acquire)
                    as u64,
                accounting_overflow: true,
                ..RelationalBranchRootReclamationOutcome::default()
            },
        }
    }
}

impl crate::runtime::RelationalRuntime {
    /// Explicit cold index maintenance. An in-flight cutover or unsettled
    /// candidate leaves the catalog untouched for a later pass.
    pub fn run_index_generation_reclamation_pass(&self) -> Option<usize> {
        self.history
            .canonical_checkpoint_gate()
            .with_settled_publication_snapshot(|| {
                // The owner inventory, rather than Arc payload counts, decides
                // which historical versions still have live obligations.
                self.history
                    .reclaim_retired_branch_roots(self.config.storage.retention.reclaim_batch_size);
                let retained = self.history.retained_index_roots();
                self.indexes.reclaim_except_versions(&retained)
            })
    }

    /// Reclaim cold immutable branch roots through the history owner. This
    /// bounded maintenance pass never walks live references or commit history.
    pub fn run_branch_root_reclamation_pass(&self) -> RelationalBranchRootReclamationOutcome {
        self.history
            .reclaim_retired_branch_roots(self.config.storage.retention.reclaim_batch_size)
    }

    /// Reclaim cold immutable branch roots, refusing any unrepresentable
    /// reclamation counter or byte total with a typed overflow instead of saturating.
    pub fn try_run_branch_root_reclamation_pass(
        &self,
    ) -> Result<RelationalBranchRootReclamationOutcome, StorageAllocationOverflow> {
        self.history
            .try_reclaim_retired_branch_roots(self.config.storage.retention.reclaim_batch_size)
    }

    /// Retired branch roots awaiting reclamation now.
    pub fn retired_branch_root_count(&self) -> usize {
        self.history.retired_branch_root_count()
    }
}

impl RelationalBranchRootReclamationOutcome {
    pub(crate) const fn new(
        roots_examined: u64,
        roots_reclaimed: u64,
        roots_still_retained: u64,
        unique_authoritative_bytes_reclaimed: u64,
    ) -> Self {
        Self {
            roots_examined,
            roots_reclaimed,
            roots_still_retained,
            unique_authoritative_bytes_reclaimed,
            accounting_overflow: false,
        }
    }

    pub const fn roots_examined(self) -> u64 {
        self.roots_examined
    }

    pub const fn roots_reclaimed(self) -> u64 {
        self.roots_reclaimed
    }

    pub const fn roots_still_retained(self) -> u64 {
        self.roots_still_retained
    }

    pub const fn unique_authoritative_bytes_reclaimed(self) -> u64 {
        self.unique_authoritative_bytes_reclaimed
    }

    pub const fn checked_unique_authoritative_bytes_reclaimed(
        self,
    ) -> Result<u64, StorageAllocationOverflow> {
        if self.accounting_overflow {
            Err(StorageAllocationOverflow)
        } else {
            Ok(self.unique_authoritative_bytes_reclaimed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::cost_counters::{try_record_reclamation, RelationalRetentionCostCounters};
    use crate::storage::substrate::StorageAllocationOverflow;

    #[test]
    fn reclamation_counters_refuse_overflow_without_saturating_or_mutating() {
        let mut counters = RelationalRetentionCostCounters {
            reclamation_roots_examined: 2,
            reclamation_roots_reclaimed: 2,
            reclamation_unique_authoritative_bytes: u64::MAX - 10,
            ..Default::default()
        };
        assert_eq!(
            try_record_reclamation(&mut counters, 1, 1, 11),
            Err(StorageAllocationOverflow)
        );
        assert_eq!(counters.reclamation_roots_examined, 2);
        assert_eq!(counters.reclamation_roots_reclaimed, 2);
        assert_eq!(counters.reclamation_unique_authoritative_bytes, u64::MAX - 10);

        assert_eq!(try_record_reclamation(&mut counters, 1, 1, 10), Ok(()));
        assert_eq!(counters.reclamation_roots_examined, 3);
        assert_eq!(counters.reclamation_roots_reclaimed, 3);
        assert_eq!(counters.reclamation_unique_authoritative_bytes, u64::MAX);
    }
}
