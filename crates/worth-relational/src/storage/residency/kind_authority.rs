//! Root-qualified identity of the region proofs admitted from its manifest.
//! Member admission verifies the region's own kind commitment; it never walks
//! this directory to acquire another region's payload.
use super::RelationalRegionResidency;
use crate::identity::data::PartitionId;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use std::sync::{Arc, OnceLock};

#[derive(Debug, Default)]
pub(crate) struct RelationalRegionKindAuthority {
    regions:
        OnceLock<worth_execution::ExecutionOrderedMap<PartitionId, super::cell::RegionCellWeak>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl RelationalRegionKindAuthority {
    pub(super) fn require_cell(&self, cell: &RelationalRegionResidency) -> Result<(), Cause> {
        if let Some(regions) = self.regions.get() {
            let expected = regions
                .get(&cell.partition_id)
                .ok_or(Cause::IntegrityMismatch)?;
            if !std::ptr::eq(expected.as_ptr(), cell) {
                return Err(Cause::IntegrityMismatch);
            }
        }
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn bind(
        &self,
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        regions: impl IntoIterator<Item = (PartitionId, Arc<RelationalRegionResidency>)>,
    ) -> Result<(), Cause> {
        let mut directory = worth_execution::ExecutionOrderedMap::default();
        for (id, cell) in regions {
            if directory.contains_key(&id) {
                return Err(Cause::Damaged);
            }
            directory
                .insert_cloned_native_in(
                    &id,
                    &RelationalRegionResidency::downgrade(&cell),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .map_err(|_| Cause::Damaged)?;
        }
        self.regions.set(directory).map_err(|_| Cause::Damaged)
    }

    pub(crate) fn for_request(
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, crate::durability::data::RelationalRecoveryStop> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .map_err(|_| {
                crate::durability::checkpoints::region_directory::corrupt(
                    "kind authority layout overflow",
                )
            })?
            .0
            .pad_to_align();
        let memory = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(|cause| {
                crate::durability::data::RelationalRecoveryStop::Request(cause.into())
            })?;
        Ok(Arc::new(Self {
            regions: Default::default(),
            _memory: memory,
        }))
    }
    pub(crate) fn bind_for_request(
        &self,
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        regions: impl IntoIterator<Item = (PartitionId, Arc<RelationalRegionResidency>)>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::durability::data::RelationalRecoveryStop> {
        let mut rows = worth_execution::ExecutionOrderedMap::default();
        for (id, cell) in regions {
            crate::durability::artifact_payload_emission::request::account_work(request, 1)?;
            if rows.contains_key(&id) {
                return Err(crate::durability::checkpoints::region_directory::corrupt(
                    "duplicate kind authority region",
                )
                .into());
            }
            rows.insert_cloned_native_for_request(
                &id,
                &RelationalRegionResidency::downgrade(&cell),
                *request,
            )
            .map_err(crate::durability::checkpoint_history::capture::map_stop)?;
        }
        self.regions.set(rows).map_err(|_| {
            crate::durability::checkpoints::region_directory::corrupt(
                "kind authority already bound",
            )
            .into()
        })
    }
}
