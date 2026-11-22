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
    #[cfg(not(test))]
    fn lock_reclamation_gate(&self) -> std::sync::MutexGuard<'_, ()> {
        self.inner
            .reclamation_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[cfg(test)]
    fn lock_reclamation_gate(&self) -> std::sync::MutexGuard<'_, ()> {
        match self.inner.reclamation_gate.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => {
                if let Some(signal) = self
                    .inner
                    .reclamation_gate_contended
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .take()
                {
                    let _ = signal.send(());
                }
                self.inner
                    .reclamation_gate
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
            }
        }
    }

    fn reclaim_retired_roots_checked(
        &self,
        maximum_roots: usize,
    ) -> Result<
        RelationalBranchRootReclamationOutcome,
        (
            RelationalBranchRootReclamationOutcome,
            StorageAllocationOverflow,
        ),
    > {
        let mut examined = 0_u64;
        let mut reclaimed = 0_u64;
        let mut unique_bytes = 0_u64;
        let pass_limit = maximum_roots.min(self.inner.retired_root_order.len());
        for _ in 0..pass_limit {
            let Some((root_key, generation)) = self.inner.retired_root_order.pop() else {
                break;
            };
            // Serialize measurement + removal through actual `drop(root)` so no
            // concurrent pass can observe an already-removed root as an external
            // holder of shared regions/backing, without holding `retired_roots`
            // or `maintenance_counters` across the drop itself.
            let _reclaim_guard = self.lock_reclamation_gate();
            enum StepOutcome {
                Reclaimed {
                    retired: super::owner::RelationalRetiredBranchRoot,
                    next_examined: u64,
                    next_reclaimed: u64,
                    next_unique_bytes: u64,
                },
                Retained {
                    next_examined: u64,
                },
                Overflow,
            }
            let step = {
                let mut roots = self.inner.write_retired_roots();
                let mut counters = super::owner::lock_maintenance_counters(&self.inner);
                let entry = roots.get(&root_key);
                let still_retired =
                    entry.is_some_and(|retired| retired.generation == generation && retired.retired);
                let removable = still_retired
                    && entry.is_some_and(|retired| {
                        retired.reservations == 0 && Arc::strong_count(&retired.root) == 1
                    });
                if removable {
                    let root_bytes = roots[&root_key]
                        .root
                        .reclaimable_unique_authoritative_bytes();
                    let next = root_bytes.ok().and_then(|bytes| {
                        Some((
                            bytes,
                            examined.checked_add(1)?,
                            reclaimed.checked_add(1)?,
                            unique_bytes.checked_add(bytes)?,
                        ))
                    });
                    if let Some((bytes, next_examined, next_reclaimed, next_unique_bytes)) = next
                        && super::cost_counters::try_record_reclamation(
                            &mut counters,
                            1,
                            1,
                            bytes,
                        )
                        .is_ok()
                    {
                        let retired = roots
                            .remove(&root_key)
                            .expect(/* Proof: `removable` verified `root_key` is present while holding the exclusive `roots` write lock. */ "removable retired root remains present");
                        StepOutcome::Reclaimed {
                            retired,
                            next_examined,
                            next_reclaimed,
                            next_unique_bytes,
                        }
                    } else {
                        self.inner.retired_root_order.push((root_key, generation));
                        StepOutcome::Overflow
                    }
                } else if let Some(next_examined) = examined.checked_add(1)
                    && super::cost_counters::try_record_reclamation(&mut counters, 1, 0, 0).is_ok()
                {
                    if still_retired {
                        self.inner.retired_root_order.push((root_key, generation));
                    }
                    StepOutcome::Retained { next_examined }
                } else {
                    if still_retired {
                        self.inner.retired_root_order.push((root_key, generation));
                    }
                    StepOutcome::Overflow
                }
            };
            match step {
                StepOutcome::Reclaimed {
                    retired,
                    next_examined,
                    next_reclaimed,
                    next_unique_bytes,
                } => {
                    let root = retired.root;
                    self.inner
                        .retired_root_locators
                        .remove(&root, root_key, generation);
                    super::owner::release_retirement_slot(&self.inner);
                    self.inner
                        .retired_root_count
                        .fetch_sub(1, std::sync::atomic::Ordering::Release);
                    examined = next_examined;
                    reclaimed = next_reclaimed;
                    unique_bytes = next_unique_bytes;
                    #[cfg(test)]
                    {
                        let pause = self
                            .inner
                            .reclamation_drop_pause
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .take();
                        if let Some(pause) = pause {
                            pause.wait();
                        }
                    }
                    drop(root);
                }
                StepOutcome::Retained { next_examined } => {
                    examined = next_examined;
                }
                StepOutcome::Overflow => {
                    let roots_still_retained = self
                        .inner
                        .retired_root_count
                        .load(std::sync::atomic::Ordering::Acquire)
                        as u64;
                    return Err((
                        RelationalBranchRootReclamationOutcome {
                            roots_examined: examined,
                            roots_reclaimed: reclaimed,
                            roots_still_retained,
                            unique_authoritative_bytes_reclaimed: unique_bytes,
                            accounting_overflow: true,
                        },
                        StorageAllocationOverflow,
                    ));
                }
            }
        }

        let roots_still_retained = self
            .inner
            .retired_root_count
            .load(std::sync::atomic::Ordering::Acquire) as u64;
        Ok(RelationalBranchRootReclamationOutcome::new(
            examined,
            reclaimed,
            roots_still_retained,
            unique_bytes,
        ))
    }

    pub(crate) fn try_reclaim_retired_roots(
        &self,
        maximum_roots: usize,
    ) -> Result<RelationalBranchRootReclamationOutcome, StorageAllocationOverflow> {
        self.reclaim_retired_roots_checked(maximum_roots)
            .map_err(|(_, overflow)| overflow)
    }

    pub(crate) fn reclaim_retired_roots(
        &self,
        maximum_roots: usize,
    ) -> RelationalBranchRootReclamationOutcome {
        match self.reclaim_retired_roots_checked(maximum_roots) {
            Ok(outcome) => outcome,
            Err((partial, StorageAllocationOverflow)) => partial,
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
