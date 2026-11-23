//! Counted leases share the sealed partition allocation across runtime threads.
use super::{PartitionState, ResidentRegion};
use std::sync::Arc;

pub(in crate::storage::residency) type SharedResidentRegion = Arc<dyn ResidentRegionBacking>;

pub(in crate::storage::residency) trait ResidentRegionBacking:
    std::fmt::Debug + Send + Sync
{
    fn resident(&self) -> &ResidentRegion;
    fn into_partition(self: Arc<Self>) -> PartitionState;
}

impl ResidentRegionBacking for ResidentRegion {
    fn resident(&self) -> &ResidentRegion {
        self
    }

    fn into_partition(self: Arc<Self>) -> PartitionState {
        match Arc::try_unwrap(self) {
            Ok(region) => region.partition,
            Err(region) => region.partition.clone(),
        }
    }
}

impl std::ops::Deref for dyn ResidentRegionBacking {
    type Target = ResidentRegion;

    fn deref(&self) -> &Self::Target {
        self.resident()
    }
}
