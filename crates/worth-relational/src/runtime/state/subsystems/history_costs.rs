/// Cost evidence owned by the history/branch publication subsystem. The
/// public inspection facade projects this type read-only; it cannot mint or
/// mutate branch authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RelationalBranchSharingCostCounters {
    pub branch_population_scans: u64,
    pub snapshot_root_reads: u64,
    pub transaction_validation_attempts: u64,
    pub retained_history_head_lookups: u64,
    pub candidate_preparations: u64,
    pub candidate_discards: u64,
    pub publication_attempts: u64,
    pub shared_root_acquisitions: u64,
    pub copied_truth_bytes: u64,
    pub copied_commit_envelopes: u64,
    pub fork_materialized_entity_count: u64,
    pub fork_materialized_relation_count: u64,
    pub fork_materialized_authoritative_bytes: u64,
    pub publication_touched_region_count: u64,
    pub publication_reused_region_count: u64,
    pub publication_persistent_index_path_nodes: u64,
    /// Changed record/history/membership/bitset values encoded by root publication.
    /// Unchanged retained values contribute zero; digest index nodes are not values.
    pub publication_content_values_hashed: u64,
    pub publication_new_authoritative_bytes: u64,
    pub reclaimable_unique_bytes: u64,
}

impl RelationalBranchSharingCostCounters {
    pub(crate) fn saturating_delta_since(self, baseline: Self) -> Self {
        Self {
            branch_population_scans: delta(
                self.branch_population_scans,
                baseline.branch_population_scans,
            ),
            snapshot_root_reads: delta(self.snapshot_root_reads, baseline.snapshot_root_reads),
            transaction_validation_attempts: delta(
                self.transaction_validation_attempts,
                baseline.transaction_validation_attempts,
            ),
            retained_history_head_lookups: delta(
                self.retained_history_head_lookups,
                baseline.retained_history_head_lookups,
            ),
            candidate_preparations: delta(
                self.candidate_preparations,
                baseline.candidate_preparations,
            ),
            candidate_discards: delta(self.candidate_discards, baseline.candidate_discards),
            publication_attempts: delta(self.publication_attempts, baseline.publication_attempts),
            shared_root_acquisitions: delta(
                self.shared_root_acquisitions,
                baseline.shared_root_acquisitions,
            ),
            copied_truth_bytes: delta(self.copied_truth_bytes, baseline.copied_truth_bytes),
            copied_commit_envelopes: delta(
                self.copied_commit_envelopes,
                baseline.copied_commit_envelopes,
            ),
            fork_materialized_entity_count: delta(
                self.fork_materialized_entity_count,
                baseline.fork_materialized_entity_count,
            ),
            fork_materialized_relation_count: delta(
                self.fork_materialized_relation_count,
                baseline.fork_materialized_relation_count,
            ),
            fork_materialized_authoritative_bytes: delta(
                self.fork_materialized_authoritative_bytes,
                baseline.fork_materialized_authoritative_bytes,
            ),
            publication_touched_region_count: delta(
                self.publication_touched_region_count,
                baseline.publication_touched_region_count,
            ),
            publication_reused_region_count: delta(
                self.publication_reused_region_count,
                baseline.publication_reused_region_count,
            ),
            publication_persistent_index_path_nodes: delta(
                self.publication_persistent_index_path_nodes,
                baseline.publication_persistent_index_path_nodes,
            ),
            publication_new_authoritative_bytes: delta(
                self.publication_new_authoritative_bytes,
                baseline.publication_new_authoritative_bytes,
            ),
            publication_content_values_hashed: delta(
                self.publication_content_values_hashed,
                baseline.publication_content_values_hashed,
            ),
            reclaimable_unique_bytes: delta(
                self.reclaimable_unique_bytes,
                baseline.reclaimable_unique_bytes,
            ),
        }
    }
}

const fn delta(current: u64, baseline: u64) -> u64 {
    current.saturating_sub(baseline)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct RelationalForkMaterializationCost {
    pub(crate) entity_count: u64,
    pub(crate) relation_count: u64,
    pub(crate) authoritative_bytes: u64,
    pub(crate) copied_commit_envelopes: u64,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RelationalPhase4ReferenceCostOwner {
    counters: std::sync::Arc<RelationalPhase4ReferenceCostCountersOwner>,
}

#[derive(Debug, Default)]
struct RelationalPhase4ReferenceCostCountersOwner {
    branch_cell_lookups: std::sync::atomic::AtomicU64,
    reference_allocations: std::sync::atomic::AtomicU64,
    branch_cell_contacts: std::sync::atomic::AtomicU64,
}

impl RelationalPhase4ReferenceCostOwner {
    pub(crate) fn detached_owner_snapshot(&self) -> Self {
        let snapshot = self.snapshot();
        Self {
            counters: std::sync::Arc::new(RelationalPhase4ReferenceCostCountersOwner {
                branch_cell_lookups: snapshot.branch_cell_lookups.into(),
                reference_allocations: snapshot.reference_allocations.into(),
                branch_cell_contacts: snapshot.branch_cell_contacts.into(),
            }),
        }
    }

    pub(crate) fn record_branch_cell_lookup(&self) {
        increment(&self.counters.branch_cell_lookups);
    }

    pub(crate) fn record_reference_allocation(&self) {
        increment(&self.counters.reference_allocations);
    }

    pub(crate) fn record_branch_cell_contact(&self) {
        increment(&self.counters.branch_cell_contacts);
    }

    pub(crate) fn snapshot(&self) -> super::RelationalPhase4ReferenceCostCounters {
        use std::sync::atomic::Ordering;
        super::RelationalPhase4ReferenceCostCounters {
            branch_cell_lookups: self.counters.branch_cell_lookups.load(Ordering::Relaxed),
            reference_allocations: self.counters.reference_allocations.load(Ordering::Relaxed),
            branch_cell_contacts: self.counters.branch_cell_contacts.load(Ordering::Relaxed),
            ..Default::default()
        }
    }
}

fn increment(counter: &std::sync::atomic::AtomicU64) {
    counter
        .fetch_update(
            std::sync::atomic::Ordering::Relaxed,
            std::sync::atomic::Ordering::Relaxed,
            |current| Some(current.saturating_add(1)),
        )
        .expect("saturating counter update cannot fail");
}
