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
        kind_indexes: Arc<super::super::kind_incremental::RegionKindIndexes>,
        requirements: Option<super::super::required_symbols::CapturedSymbolRequirements>,
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
            .map_err(|cause| RegionContentFailure::Execution(cause.into()))?;
        let control_memory = request
            .reserve_retained_memory(control_bytes)
            .map_err(|cause| RegionContentFailure::Execution(cause.into()))?;
        let control_memory = control_memory.map(Arc::new);
        let region = Arc::new(ResidentRegion {
            symbol_requirements: requirements
                .map(|value| OnceLock::from(Arc::new(value)))
                .unwrap_or_default(),
            history_presence: super::super::history_presence::HistoryPresence::complete(),
            kind_indexes: Some(kind_indexes),
            partition,
            commitment: Some(commitment),
            _history_custody: None,
            _memory: memory,
        });
        let cell = Arc::new(Self {
            partition_id: region.partition.partition_id,
            checkpoint_bindings: Mutex::new(Default::default()),
            backing: None,
            head_readmission: None,
            raw_head: Mutex::new(None),
            raw_full: Mutex::new(None),
            state: Mutex::new(RegionState::new(ResidencyState::Full(region.clone()), None)),
            damage: OnceLock::new(),
            symbol_aliases: OnceLock::new(),
            symbol_inheritance: OnceLock::new(),
            leases: AtomicUsize::new(1),
            _control_memory: control_memory,
        });
        Ok(RelationalResidencyLease::accepted(cell, region))
    }
}
