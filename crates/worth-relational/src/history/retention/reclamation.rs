use std::sync::Arc;

/// Bounded maintenance result for retired immutable branch roots.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RelationalBranchRootReclamationOutcome {
    roots_examined: u64,
    roots_reclaimed: u64,
    roots_still_retained: u64,
    unique_authoritative_bytes_reclaimed: u64,
}

impl super::RelationalBranchRetentionOwner {
    pub(crate) fn reclaim_retired_roots(
        &self,
        maximum_roots: usize,
    ) -> RelationalBranchRootReclamationOutcome {
        let mut examined = 0_u64;
        let mut reclaimed = 0_u64;
        let mut unique_bytes = 0_u64;
        let pass_limit = maximum_roots.min(self.inner.retired_root_order.len());
        for _ in 0..pass_limit {
            let Some((root_key, generation)) = self.inner.retired_root_order.pop() else {
                break;
            };
            examined = examined.saturating_add(1);
            let reclaimable = self
                .inner
                .retired_roots
                .remove_if(&root_key, |_, retired| {
                    retired.generation == generation
                        && retired.retired
                        && retired.reservations == 0
                        && retired
                            .root
                            .as_ref()
                            .is_some_and(|root| Arc::strong_count(root) == 1)
                })
                .map(|(_, retired)| retired);
            if let Some(retired) = reclaimable {
                let root = retired
                    .root
                    .expect("retired inventory entry owns its exact root");
                if let Some(commit_id) = root.commit_id() {
                    remove_commit_root(&self.inner, commit_id, root_key, generation);
                }
                super::owner::release_retirement_slot(&self.inner);
                self.inner
                    .retired_root_count
                    .fetch_sub(1, std::sync::atomic::Ordering::Release);
                unique_bytes =
                    unique_bytes.saturating_add(root.reclaimable_unique_authoritative_bytes());
                reclaimed = reclaimed.saturating_add(1);
                drop(root);
            } else if self
                .inner
                .retired_roots
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
        super::cost_counters::record_reclamation(
            &mut super::owner::lock_maintenance_counters(&self.inner),
            examined,
            reclaimed,
            unique_bytes,
        );
        RelationalBranchRootReclamationOutcome::new(
            examined,
            reclaimed,
            roots_still_retained,
            unique_bytes,
        )
    }
}

fn remove_commit_root(
    owner: &super::owner::RelationalBranchRetentionOwnerInner,
    commit_id: crate::history::data::CommitId,
    root_key: usize,
    generation: u64,
) {
    owner
        .retired_roots_by_commit
        .remove_if(&commit_id, |_, indexed| *indexed == (root_key, generation));
}

impl crate::runtime::RelationalRuntime {
    /// Explicit cold index maintenance. An in-flight cutover or unsettled
    /// candidate leaves the catalog untouched for a later pass.
    pub fn run_index_generation_reclamation_pass(&self) -> Option<usize> {
        self.history
            .canonical_checkpoint_gate()
            .with_quiescent_index_reclamation(|| {
                // The owner inventory, rather than Arc payload counts, decides
                // which historical versions still have live obligations.
                self.history.reclaim_retired_branch_roots(usize::MAX);
                let retained = self.history.retained_index_versions();
                self.indexes.reclaim_except_versions(&retained)
            })
    }

    /// Reclaim cold immutable branch roots through the history owner. This
    /// bounded maintenance pass never walks live references or commit history.
    pub fn run_branch_root_reclamation_pass(&mut self) -> RelationalBranchRootReclamationOutcome {
        let outcome = self
            .history
            .reclaim_retired_branch_roots(self.config.storage.retention.reclaim_batch_size);
        self.run_index_generation_reclamation_pass();
        outcome
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
}
