//! Exact current routing with the calls actually made again excluded.
use super::super::routing::ComputationPartitionRouting;
use std::{collections::BTreeSet, sync::Arc};
use worth_foundational::facade::PartitionIdentity;

pub(super) struct CarriedPartitions {
    routing: Arc<ComputationPartitionRouting>,
    fresh: BTreeSet<PartitionIdentity>,
}
impl CarriedPartitions {
    pub(super) fn new(
        routing: Arc<ComputationPartitionRouting>,
        fresh: BTreeSet<PartitionIdentity>,
    ) -> Self {
        Self { routing, fresh }
    }
    pub(super) fn contains(&self, id: &PartitionIdentity) -> bool {
        self.routing.digest(*id).is_some() && !self.fresh.contains(id)
    }
    pub(super) fn iter(&self) -> impl Iterator<Item = PartitionIdentity> + '_ {
        // Full read-closure verification deliberately remains visible here.
        self.routing
            .partitions()
            .filter(|id| !self.fresh.contains(id))
    }
}
