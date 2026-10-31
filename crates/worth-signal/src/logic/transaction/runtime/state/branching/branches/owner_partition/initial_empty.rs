use super::{SignalBranchId, SignalOwnerMetadataState, SignalOwnerPartition};

impl<D, I, T> SignalOwnerMetadataState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn has_initial_empty_history(&self) -> bool {
        self.next_snapshot_id == 0
            && self.snapshots.is_empty()
            && self.children_by_parent.is_empty()
            && self.retirement_reservations.is_empty()
            && self.retirement_receipts.is_empty()
            && self.active_merge_participants.is_empty()
            && self.reserved_retirement_receipt_count == 0
    }
}

impl<D, I, T> SignalOwnerPartition<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn initial_empty_branch(&self) -> Option<SignalBranchId> {
        let [(handle, state, generation, restore)] = self.cells.as_slice() else {
            return None;
        };
        (self.next_branch_id == 1
            && self.metadata.has_initial_empty_history()
            && handle.id == self.selected_branch_id
            && handle.head_snapshot_id.is_none()
            && *generation == 0
            && restore.is_none()
            && state.has_initial_empty_origin())
        .then_some(handle.id)
    }
}
