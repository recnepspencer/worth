mod fork_growth;
mod reserved_fork;
use crate::data::dependency::{DependencySnapshotShapeStore, DependencySnapshotStore};
use crate::data::graph::{DependencyEdgeStore, ReverseSubscriptionIndex, SubscriberEdgeStore};
use crate::data::handle::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetainedTopologyIndexDenial {
    Snapshots(crate::data::dependency::DependencySnapshotIndexDenial),
    DependencyEdgesRequireReconstruction,
    SubscriberEdgesRequireReconstruction,
    ReverseSubscriptionsRequireReconstruction,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct EdgeTopology {
    #[serde(default)]
    pub(in crate::data::graph) dependency_snapshots: DependencySnapshotStore,
    #[serde(default)]
    pub(in crate::data::graph) dependency_snapshot_shapes: DependencySnapshotShapeStore,
    #[serde(skip, default)]
    pub(in crate::data::graph) dependency_snapshot_storage_custody: Option<
        std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionReservation>,
    >,
    #[serde(default)]
    pub(in crate::data::graph) dependency_edges: DependencyEdgeStore,
    #[serde(default)]
    pub(in crate::data::graph) subscriber_edges: SubscriberEdgeStore,
    #[serde(skip, default)]
    pub(in crate::data::graph) reverse_subscriptions: ReverseSubscriptionIndex,
    #[serde(skip, default)]
    pub(in crate::data::graph) pending_revalidation_waiters:
        crate::data::persistent_ord_map::PersistentOrdMap<NodeId, im::OrdSet<NodeId>>,
    #[serde(skip, default)]
    pub(in crate::data::graph) pending_revalidation_storage_custody: Option<
        std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionReservation>,
    >,
}

impl EdgeTopology {
    /// Retained execution must not repair indexes on its ordinary lane.
    /// These owner readiness checks do not establish source or store affinity.
    pub(crate) fn require_retained_indexes(&self) -> Result<(), RetainedTopologyIndexDenial> {
        self.dependency_snapshots
            .require_retained_indexes(&self.dependency_snapshot_shapes)
            .map_err(RetainedTopologyIndexDenial::Snapshots)?;
        if self
            .dependency_edges
            .retained_interner_requires_reconstruction()
        {
            return Err(RetainedTopologyIndexDenial::DependencyEdgesRequireReconstruction);
        }
        if self
            .subscriber_edges
            .retained_interner_requires_reconstruction()
        {
            return Err(RetainedTopologyIndexDenial::SubscriberEdgesRequireReconstruction);
        }
        if !self.reverse_subscriptions.is_valid() {
            return Err(RetainedTopologyIndexDenial::ReverseSubscriptionsRequireReconstruction);
        }
        Ok(())
    }

    pub(crate) fn fork_persistent(&mut self) -> Self {
        Self {
            dependency_snapshots: self.dependency_snapshots.fork_persistent(),
            dependency_snapshot_shapes: self.dependency_snapshot_shapes.fork_persistent(),
            dependency_snapshot_storage_custody: self.dependency_snapshot_storage_custody.clone(),
            dependency_edges: self.dependency_edges.fork_persistent(),
            subscriber_edges: self.subscriber_edges.fork_persistent(),
            reverse_subscriptions: self.reverse_subscriptions.fork_persistent(),
            pending_revalidation_waiters: self.pending_revalidation_waiters.fork_persistent(),
            pending_revalidation_storage_custody: self.pending_revalidation_storage_custody.clone(),
        }
    }

    pub(crate) fn operational_clone(&self) -> Self {
        Self {
            dependency_snapshots: self.dependency_snapshots.operational_clone(),
            dependency_snapshot_shapes: self.dependency_snapshot_shapes.operational_clone(),
            dependency_snapshot_storage_custody: None,
            dependency_edges: self.dependency_edges.operational_clone(),
            subscriber_edges: self.subscriber_edges.operational_clone(),
            reverse_subscriptions: self.reverse_subscriptions.operational_clone(),
            pending_revalidation_waiters: self.pending_revalidation_waiters.operational_clone(),
            pending_revalidation_storage_custody: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        Self {
            dependency_snapshots: self.dependency_snapshots.fork_storage_identity(),
            dependency_snapshot_shapes: self.dependency_snapshot_shapes.fork_storage_identity(),
            dependency_snapshot_storage_custody: self.dependency_snapshot_storage_custody.clone(),
            dependency_edges: self.dependency_edges.fork_storage_identity(),
            subscriber_edges: self.subscriber_edges.fork_storage_identity(),
            reverse_subscriptions: self.reverse_subscriptions.fork_storage_identity(),
            pending_revalidation_waiters: self.pending_revalidation_waiters.fork_storage_identity(),
            pending_revalidation_storage_custody: self.pending_revalidation_storage_custody.clone(),
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        self.dependency_snapshots
            .shares_storage_with(&other.dependency_snapshots)
            && self
                .dependency_snapshot_shapes
                .shares_storage_with(&other.dependency_snapshot_shapes)
            && self
                .dependency_edges
                .shares_storage_with(&other.dependency_edges)
            && self
                .subscriber_edges
                .shares_storage_with(&other.subscriber_edges)
            && self
                .reverse_subscriptions
                .shares_storage_with(&other.reverse_subscriptions)
            && self
                .pending_revalidation_waiters
                .ptr_eq(&other.pending_revalidation_waiters)
    }
}
mod retained_charge;
