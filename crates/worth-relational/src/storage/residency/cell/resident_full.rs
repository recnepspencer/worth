//! Admission of the two real Full residency allocations before either grows.
use super::*;
use crate::storage::residency::RegionContentFailure;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

fn arc_bytes<T>() -> Option<u64> {
    let layout = std::alloc::Layout::new::<(usize, usize)>()
        .extend(std::alloc::Layout::new::<T>())
        .ok()?
        .0
        .pad_to_align();
    u64::try_from(layout.size()).ok()
}

impl RelationalRegionResidency {
    pub(in crate::storage::residency) fn resident_for_request(
        partition: PartitionState,
        commitment: PartitionContentCommitment,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalResidencyLease<Full>, RegionContentFailure> {
        let overflow = || {
            RegionContentFailure::Content(
                crate::storage::overlay::PartitionContentDigestError::AllocationAccountingOverflow,
            )
        };
        let region_bytes = arc_bytes::<ResidentRegion>().ok_or_else(overflow)?;
        let control_bytes = arc_bytes::<Self>()
            .and_then(|cell| cell.checked_add(arc_bytes::<ExecutionMemoryReservation>()?))
            .ok_or_else(overflow)?;
        let memory = request
            .reserve_retained_memory(region_bytes)
            .map_err(|cause| RegionContentFailure::Request(cause.into()))?;
        let control_memory = request
            .reserve_retained_memory(control_bytes)
            .map_err(|cause| RegionContentFailure::Request(cause.into()))?;
        let control_memory = control_memory.map(Arc::new);
        let region = Arc::new(ResidentRegion {
            partition,
            commitment: Some(commitment),
            _memory: memory,
        });
        let cell = Arc::new(Self {
            partition_id: region.partition.partition_id,
            checkpoint_bindings: Mutex::new(Default::default()),
            backing: None,
            head_readmission: None,
            _backing_custody: None,
            raw_head: Mutex::new(None),
            raw_full: Mutex::new(None),
            state: Mutex::new(ResidencyState::Full(region.clone())),
            damage: OnceLock::new(),
            leases: AtomicUsize::new(1),
            _control_memory: control_memory,
        });
        Ok(RelationalResidencyLease::accepted(cell, region))
    }
}

#[cfg(test)]
mod tests;
