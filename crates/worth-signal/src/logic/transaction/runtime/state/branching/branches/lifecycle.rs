use crate::state::{SignalBranchId, SignalSnapshotId};

use crate::branch::SignalBranchRetirementReceipt;

use super::catalog::BranchManager;

pub(in crate::logic::transaction::runtime) struct BranchRetirementReclaimedBreadth {
    pub branch_state_count: u32,
    pub snapshot_state_count: u32,
    pub runtime_meta_count: u32,
}

impl<D, I, T> BranchManager<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn retire_stored_branch(
        &mut self,
        branch_id: SignalBranchId,
    ) -> Option<BranchRetirementReclaimedBreadth> {
        let branch_state = self.branches.remove(&branch_id)?;
        let parent_branch_id = branch_state.ancestry().parent_branch_id();
        let snapshot_keys = self
            .snapshots
            .range((branch_id, SignalSnapshotId(0))..=(branch_id, SignalSnapshotId(u64::MAX)))
            .map(|(key, _)| *key)
            .collect::<Vec<_>>();
        for key in &snapshot_keys {
            self.snapshots.remove(key);
        }
        let runtime_meta_count = u32::from(self.branch_meta.remove(&branch_id).is_some());
        self.children_by_parent.remove(&branch_id);
        if let Some(parent_branch_id) = parent_branch_id {
            let mut remove_parent_entry = false;
            if let Some(children) = self.children_by_parent.get_mut(&parent_branch_id) {
                children.remove(&branch_id);
                remove_parent_entry = children.is_empty();
            }
            if remove_parent_entry {
                self.children_by_parent.remove(&parent_branch_id);
            }
        }
        self.branch_head_generations.remove(&branch_id);
        self.branch_restore_snapshot_ids.remove(&branch_id);
        self.live_branch_catalog.remove(&branch_id);
        Some(BranchRetirementReclaimedBreadth {
            branch_state_count: 1,
            snapshot_state_count: snapshot_keys.len() as u32,
            runtime_meta_count,
        })
    }

    pub fn retain_retirement_receipt(&mut self, receipt: SignalBranchRetirementReceipt) {
        self.retirement_receipts
            .insert(receipt.retired_branch().id, receipt);
    }

    pub fn branch_retirement_receipt(
        &self,
        branch_id: SignalBranchId,
    ) -> Option<&SignalBranchRetirementReceipt> {
        self.retirement_receipts.get(&branch_id)
    }
}
