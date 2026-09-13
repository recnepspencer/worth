use super::DependencySnapshotStore;
use crate::data::retained_storage::SignalConditionalRetentionReservation;

impl DependencySnapshotStore {
    pub(crate) fn fork_reserved(
        &mut self,
        resources: &mut SignalConditionalRetentionReservation,
    ) -> Self {
        Self {
            snapshots: self.snapshots.fork_reserved(resources),
            interner: self.interner.fork_reserved(resources),
            shape_handles: self.shape_handles.fork_reserved(resources),
        }
    }
}
