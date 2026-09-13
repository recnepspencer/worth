use crate::state::SignalBranchId;

use super::super::catalog::BranchManager;
use super::{SignalOwnerPartitionDenial, MAXIMUM_RETAINED_SIGNAL_BRANCH_RETIREMENT_RECEIPTS};

impl<D, I, T> BranchManager<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::logic::transaction::runtime) fn conditional_retention_budget_matches(
        &self,
        budget: crate::runtime_policy::SignalConditionalEvaluationBudget,
        temporal_budget: crate::runtime_policy::SignalConditionalTemporalBudget,
    ) -> bool {
        self.branches.values().all(|state| {
            let installed = state
                .graph()
                .installed_runtime_policy()
                .conditional_evaluation_budget();
            installed.maximum_retained_slots == budget.maximum_retained_slots
                && installed.maximum_retained_bytes == budget.maximum_retained_bytes
                && state
                    .graph()
                    .installed_runtime_policy()
                    .conditional_temporal_budget()
                    == temporal_budget
        })
    }

    pub(in crate::logic::transaction::runtime) fn validate_owner_partition(
        &self,
        active_branch_id: SignalBranchId,
        maximum_live_branches: usize,
    ) -> Result<(), SignalOwnerPartitionDenial> {
        if self.live_branch_catalog.len() > maximum_live_branches {
            return Err(SignalOwnerPartitionDenial::LiveBranchCapacityExhausted {
                maximum_live_branches,
            });
        }
        if self.retirement_receipts.len() > MAXIMUM_RETAINED_SIGNAL_BRANCH_RETIREMENT_RECEIPTS {
            return Err(
                SignalOwnerPartitionDenial::RetirementReceiptCapacityExhausted {
                    maximum_retained_receipts: MAXIMUM_RETAINED_SIGNAL_BRANCH_RETIREMENT_RECEIPTS,
                },
            );
        }
        if !self.live_branch_catalog.contains_key(&active_branch_id) {
            return Err(SignalOwnerPartitionDenial::ActiveBranchMissing);
        }
        for branch_id in self.live_branch_catalog.keys().copied() {
            if branch_id != active_branch_id && !self.branches.contains_key(&branch_id) {
                return Err(SignalOwnerPartitionDenial::StoredBranchMissing(branch_id));
            }
        }
        for branch_id in self.branches.keys().copied() {
            if branch_id == active_branch_id || !self.live_branch_catalog.contains_key(&branch_id) {
                return Err(SignalOwnerPartitionDenial::UnexpectedStoredBranch(
                    branch_id,
                ));
            }
        }
        for branch_id in self.live_branch_catalog.keys().copied() {
            if !self.branch_head_generations.contains_key(&branch_id)
                || !self.branch_restore_snapshot_ids.contains_key(&branch_id)
            {
                return Err(SignalOwnerPartitionDenial::MissingBranchHead(branch_id));
            }
        }
        if let Some(branch_id) = self
            .branch_head_generations
            .keys()
            .chain(self.branch_restore_snapshot_ids.keys())
            .copied()
            .find(|branch_id| !self.live_branch_catalog.contains_key(branch_id))
        {
            return Err(SignalOwnerPartitionDenial::UnexpectedBranchHead(branch_id));
        }
        Ok(())
    }
}
