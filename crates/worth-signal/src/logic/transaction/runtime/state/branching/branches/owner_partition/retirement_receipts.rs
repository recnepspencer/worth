use std::collections::BTreeSet;

use crate::branch::SignalBranchRetirementDenial;
use crate::state::SignalBranchId;

use super::SignalOwnerMetadataState;

pub(crate) struct SignalOwnerRetirementCleanup<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    snapshots: Vec<super::SnapshotBranchState<D, I, T>>,
}

impl<D, I, T> SignalOwnerRetirementCleanup<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn reclaimed_snapshot_count(&self) -> u32 {
        self.snapshots.len() as u32
    }

    pub(crate) fn discard(self) {
        drop(self.snapshots);
    }

    #[cfg(test)]
    pub(crate) fn discard_with_observer(self, before_payload_drop: impl FnOnce(u32)) {
        before_payload_drop(self.snapshots.len() as u32);
        drop(self.snapshots);
    }
}

impl<D, I, T> SignalOwnerMetadataState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn reserve_retirement_contract_after(
        &mut self,
        branch_id: SignalBranchId,
        retired_before: &BTreeSet<SignalBranchId>,
    ) -> Result<u32, SignalBranchRetirementDenial> {
        if !self.retirement_reservations.insert(branch_id) {
            return Err(SignalBranchRetirementDenial::RetirementInProgress { branch_id });
        }
        let children = self
            .branch_children(branch_id)
            .into_iter()
            .filter(|child_id| {
                !retired_before.contains(child_id)
                    || !self.retirement_reservations.contains(child_id)
            })
            .collect::<Vec<_>>();
        if !children.is_empty() {
            self.retirement_reservations.remove(&branch_id);
            return Err(SignalBranchRetirementDenial::LiveChildren {
                branch_id,
                child_branch_ids: children,
            });
        }
        if self.is_merge_participant(branch_id) {
            self.retirement_reservations.remove(&branch_id);
            return Err(SignalBranchRetirementDenial::MergeParticipant { branch_id });
        }
        if self.retirement_receipts.contains_key(&branch_id) {
            self.retirement_reservations.remove(&branch_id);
            return Err(SignalBranchRetirementDenial::RetiredBranch { branch_id });
        }
        if self.retirement_receipts.len() + self.reserved_retirement_receipt_count
            >= self.maximum_retirement_receipts
        {
            self.retirement_reservations.remove(&branch_id);
            return Err(
                SignalBranchRetirementDenial::RetirementReceiptCapacityExhausted {
                    maximum_retained_receipts: self.maximum_retirement_receipts,
                },
            );
        }
        self.reserved_retirement_receipt_count += 1;
        let snapshot_count = self
            .snapshots
            .range(
                (branch_id, crate::state::SignalSnapshotId(0))
                    ..=(branch_id, crate::state::SignalSnapshotId(u64::MAX)),
            )
            .count() as u32;
        Ok(snapshot_count)
    }

    pub(crate) fn cancel_retirement_contract(&mut self, branch_id: SignalBranchId) {
        let removed = self.retirement_reservations.remove(&branch_id);
        debug_assert!(removed);
        self.reserved_retirement_receipt_count = self
            .reserved_retirement_receipt_count
            .checked_sub(1)
            .expect("retirement receipt reservation releases exactly once");
    }

    pub(crate) fn complete_retirement_contract(
        &mut self,
        branch_id: SignalBranchId,
        parent_branch_id: Option<SignalBranchId>,
        receipt: crate::branch::SignalBranchRetirementReceipt,
    ) -> SignalOwnerRetirementCleanup<D, I, T> {
        let snapshot_keys = self
            .snapshots
            .range(
                (branch_id, crate::state::SignalSnapshotId(0))
                    ..=(branch_id, crate::state::SignalSnapshotId(u64::MAX)),
            )
            .map(|(key, _)| *key)
            .collect::<Vec<_>>();
        let snapshots = snapshot_keys
            .into_iter()
            .filter_map(|key| self.snapshots.remove(&key))
            .collect();
        self.children_by_parent.remove(&branch_id);
        if let Some(parent_branch_id) = parent_branch_id {
            let remove_parent = self
                .children_by_parent
                .get_mut(&parent_branch_id)
                .is_some_and(|children| {
                    children.remove(&branch_id);
                    children.is_empty()
                });
            if remove_parent {
                self.children_by_parent.remove(&parent_branch_id);
            }
        }
        self.active_merge_participants.remove(&branch_id);
        let removed = self.retirement_reservations.remove(&branch_id);
        debug_assert!(removed);
        self.reserved_retirement_receipt_count = self
            .reserved_retirement_receipt_count
            .checked_sub(1)
            .expect("performed retirement consumes one reserved receipt slot");
        self.retain_retirement_receipt(receipt);
        SignalOwnerRetirementCleanup { snapshots }
    }

    pub(crate) fn fork_parent_accepts_child(&self, parent_branch_id: SignalBranchId) -> bool {
        !self.retirement_reservations.contains(&parent_branch_id)
    }

    pub(crate) fn branch_accepts_retention_acquisition(&self, branch_id: SignalBranchId) -> bool {
        !self.retirement_reservations.contains(&branch_id)
            && !self.retirement_receipts.contains_key(&branch_id)
    }

    #[cfg(test)]
    pub(crate) fn retirement_contract_counts(&self) -> (usize, usize, usize) {
        (
            self.retirement_reservations.len(),
            self.reserved_retirement_receipt_count,
            self.retirement_receipts.len(),
        )
    }
}
