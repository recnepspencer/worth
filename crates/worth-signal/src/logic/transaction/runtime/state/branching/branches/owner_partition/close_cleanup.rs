use std::collections::BTreeSet;

use crate::state::SignalBranchId;

use super::{SignalOwnerMetadataState, SnapshotBranchState};

pub(crate) struct SignalOwnerMetadataCloseBatch<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    _snapshots: Vec<SnapshotBranchState<D, I, T>>,
    _lineages: Vec<BTreeSet<SignalBranchId>>,
    cleaned_entries: usize,
}

impl<D, I, T> SignalOwnerMetadataCloseBatch<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn cleaned_entries(&self) -> usize {
        self.cleaned_entries
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.cleaned_entries == 0
    }
}

impl<D, I, T> SignalOwnerMetadataState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn take_close_batch(
        &mut self,
        maximum_batch_size: usize,
    ) -> SignalOwnerMetadataCloseBatch<D, I, T> {
        debug_assert_eq!(self.reserved_retirement_receipt_count, 0);
        debug_assert!(self.retirement_reservations.is_empty());
        let mut snapshots = Vec::new();
        let mut lineages = Vec::new();
        let mut cleaned_entries = 0;
        while cleaned_entries < maximum_batch_size {
            if let Some((_key, snapshot)) = self.snapshots.pop_first() {
                snapshots.push(snapshot);
            } else if let Some((_branch_id, children)) = self.children_by_parent.pop_first() {
                lineages.push(children);
            } else if let Some(branch_id) = self.active_merge_participants.pop_first() {
                let _ = branch_id;
            } else {
                break;
            }
            cleaned_entries += 1;
        }
        SignalOwnerMetadataCloseBatch {
            _snapshots: snapshots,
            _lineages: lineages,
            cleaned_entries,
        }
    }
}
