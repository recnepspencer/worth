//! An owned edit consumes Full custody and freezes back into a counted cell.
use super::{Full, RelationalRegionResidency, RelationalResidencyLease};
use crate::storage::overlay::PartitionState;

pub(crate) struct ResidentFullRegionEdit {
    partition: PartitionState,
}

impl RelationalResidencyLease<Full> {
    pub(crate) fn into_edit(self) -> ResidentFullRegionEdit {
        ResidentFullRegionEdit {
            partition: self.into_owned_partition(),
        }
    }

    pub(crate) fn has_other_observers(&self) -> bool {
        std::sync::Arc::strong_count(self.cell()) > 1
    }
}

impl ResidentFullRegionEdit {
    pub(crate) fn transform(
        self,
        transform: impl FnOnce(PartitionState) -> PartitionState,
    ) -> Self {
        Self {
            partition: transform(self.partition),
        }
    }

    pub(crate) fn partition(&self) -> &PartitionState {
        &self.partition
    }

    pub(crate) fn partition_mut(&mut self) -> &mut PartitionState {
        &mut self.partition
    }

    pub(crate) fn freeze(self) -> RelationalResidencyLease<Full> {
        RelationalRegionResidency::resident_uncommitted(self.partition)
    }
}
