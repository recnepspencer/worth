use crate::data::graph::SignalGraph;
use crate::state::{SignalBranchId, SignalSnapshotId};

use super::authority::BranchState;
use super::catalog::BranchManager;
use super::{SnapshotBranchState, SnapshotStatePacket};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::logic::transaction::runtime) enum SignalBranchSnapshotStorageDenial {
    CapacityExhausted { maximum_stored_snapshots: usize },
}

impl<D, I, T> BranchManager<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn synchronize_snapshot_identity_high_water(&mut self, next_snapshot_id: u64) {
        self.next_snapshot_id = self.next_snapshot_id.max(next_snapshot_id);
    }

    pub(crate) fn snapshot_identity_available(&self) -> Result<(), SignalSnapshotId> {
        self.next_snapshot_id
            .checked_add(1)
            .map(|_| ())
            .ok_or(SignalSnapshotId(self.next_snapshot_id))
    }

    pub(crate) fn reserve_snapshot_identity(
        &mut self,
    ) -> Result<SignalSnapshotId, SignalSnapshotId> {
        let snapshot_id = SignalSnapshotId(self.next_snapshot_id);
        let next_snapshot_id = self.next_snapshot_id.checked_add(1).ok_or(snapshot_id)?;
        self.next_snapshot_id = next_snapshot_id;
        Ok(snapshot_id)
    }

    pub(crate) fn snapshot_reconstruction_runtime_is_pristine(
        &self,
        branch_id: SignalBranchId,
    ) -> bool {
        self.live_branch_catalog.len() == 1
            && self.live_branch_catalog.contains_key(&branch_id)
            && self.branches.is_empty()
            && self.snapshots.is_empty()
            && self.next_snapshot_id == 0
    }

    pub fn insert_snapshot(&mut self, packet: SnapshotStatePacket<D, I, T>) {
        debug_assert!(self.snapshots.len() < self.maximum_stored_snapshots);
        let (branch_id, snapshot_id, state) = packet.into_parts();
        self.snapshots.insert((branch_id, snapshot_id), state);
    }

    pub fn ensure_snapshot_storage_available(
        &self,
    ) -> Result<(), SignalBranchSnapshotStorageDenial> {
        if self.snapshots.len() >= self.maximum_stored_snapshots {
            return Err(SignalBranchSnapshotStorageDenial::CapacityExhausted {
                maximum_stored_snapshots: self.maximum_stored_snapshots,
            });
        }
        Ok(())
    }

    pub fn snapshot_state(
        &self,
        branch_id: SignalBranchId,
        snapshot_id: SignalSnapshotId,
    ) -> Option<&SnapshotBranchState<D, I, T>> {
        self.snapshots.get(&(branch_id, snapshot_id))
    }

    pub fn replay_graph<'a>(
        &'a self,
        branch_id: SignalBranchId,
        active_branch: SignalBranchId,
        active_graph: &'a SignalGraph,
    ) -> Option<&'a SignalGraph> {
        if branch_id == active_branch {
            Some(active_graph)
        } else {
            self.branches.get(&branch_id).map(BranchState::graph)
        }
    }
}
