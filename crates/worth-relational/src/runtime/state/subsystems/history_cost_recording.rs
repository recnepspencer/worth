use crate::branch::RelationalBranchRoot;
use crate::history::data::BranchId;

use super::{HistorySubsystem, RelationalBranchSharingCostCounters};

impl HistorySubsystem {
    pub(crate) fn record_snapshot_root_read(&self, branch_id: &BranchId) {
        self.record_branch_sharing_operation(branch_id, |costs| {
            costs.snapshot_root_reads = costs.snapshot_root_reads.saturating_add(1);
        });
    }

    pub(crate) fn record_publication_attempt(&self, branch_id: &BranchId) {
        self.record_branch_sharing_operation(branch_id, |costs| {
            costs.publication_attempts = costs.publication_attempts.saturating_add(1);
        });
    }

    pub(crate) fn record_root_publication(
        &self,
        branch_id: &BranchId,
        root: &RelationalBranchRoot,
        new_authoritative_bytes: u64,
    ) {
        let cost = root.publication_cost();
        self.record_branch_sharing_operation(branch_id, |costs| {
            costs.copied_truth_bytes = costs
                .copied_truth_bytes
                .saturating_add(cost.copied_truth_bytes);
            costs.copied_commit_envelopes = costs
                .copied_commit_envelopes
                .saturating_add(cost.copied_commit_envelopes);
            costs.publication_touched_region_count = costs
                .publication_touched_region_count
                .saturating_add(cost.touched_regions);
            costs.publication_reused_region_count = costs
                .publication_reused_region_count
                .saturating_add(cost.reused_regions);
            costs.publication_persistent_index_path_nodes = costs
                .publication_persistent_index_path_nodes
                .saturating_add(cost.persistent_index_path_nodes);
            costs.publication_new_authoritative_bytes = costs
                .publication_new_authoritative_bytes
                .saturating_add(new_authoritative_bytes);
            costs.publication_content_values_hashed = costs
                .publication_content_values_hashed
                .saturating_add(cost.content_values_hashed);
        });
    }

    pub(crate) fn sharing_costs_for_branch(
        &self,
        branch_id: &BranchId,
    ) -> RelationalBranchSharingCostCounters {
        self.branch_cell(branch_id)
            .map(|cell| cell.publication_cell().sharing_costs())
            .unwrap_or_default()
    }

    fn record_branch_sharing_operation(
        &self,
        branch_id: &BranchId,
        record: impl FnOnce(&mut RelationalBranchSharingCostCounters),
    ) {
        let Some(cell) = self.branch_cell(branch_id) else {
            return;
        };
        cell.publication_cell().record_sharing_cost(record);
    }
}
