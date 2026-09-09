use super::EdgeTopology;
use crate::data::retained_storage::SignalConditionalRetentionReservation;

impl EdgeTopology {
    pub(crate) fn fork_reserved(
        &mut self,
        resources: &mut SignalConditionalRetentionReservation,
    ) -> Self {
        Self {
            dependency_snapshots: self.dependency_snapshots.fork_reserved(resources),
            dependency_snapshot_shapes: self.dependency_snapshot_shapes.fork_reserved(resources),
            dependency_snapshot_storage_custody: self.dependency_snapshot_storage_custody.clone(),
            dependency_edges: self.dependency_edges.fork_reserved(resources),
            subscriber_edges: self.subscriber_edges.fork_reserved(resources),
            reverse_subscriptions: self.reverse_subscriptions.fork_reserved(resources),
            pending_revalidation_waiters: self
                .pending_revalidation_waiters
                .fork_reserved(resources),
            pending_revalidation_storage_custody: self.pending_revalidation_storage_custody.clone(),
        }
    }
}
