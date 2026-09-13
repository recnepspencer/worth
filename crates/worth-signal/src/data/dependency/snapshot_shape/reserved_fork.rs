use super::DependencySnapshotShapeStore;
use crate::data::retained_storage::SignalConditionalRetentionReservation;

impl DependencySnapshotShapeStore {
    pub(crate) fn fork_reserved(
        &mut self,
        resources: &mut SignalConditionalRetentionReservation,
    ) -> Self {
        Self {
            shapes: self.shapes.fork_reserved(resources),
            interner: self.interner.fork_reserved(resources),
        }
    }
}
