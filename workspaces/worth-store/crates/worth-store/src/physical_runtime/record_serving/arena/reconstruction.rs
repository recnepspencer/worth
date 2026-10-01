use super::{ExtentArenaAllocationOwner, SharedArenaAllocationOwner};
use crate::physical_runtime::record_serving::{
    access::manifest_routing::ManifestDiscoveryCounterSnapshot,
    planning::free_space_routing::FreeSpaceReader, residency::PhysicalResidencyWorkPort,
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, AdmittedRecordPlacementPolicy,
    RecordAppendDenial, RecordAppendError,
};
use std::sync::{Arc, Mutex};
use worth_store_physical_format::DurableFreeSpaceManifestHeader;

pub(in crate::physical_runtime::record_serving) fn reconstruct(
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    residency: PhysicalResidencyWorkPort,
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
    placement: AdmittedRecordPlacementPolicy,
    header: &DurableFreeSpaceManifestHeader,
) -> Result<SharedArenaAllocationOwner, RecordAppendError> {
    // Private, retained grant: this charge cannot simultaneously fund frame work.
    // 4 KiB per entry conservatively covers node slack in three BTree indexes;
    // it is reserved capacity, not an exact materialized-heap observation.
    let charge = residency
        .begin_foreground_write_operation(
            std::num::NonZeroU64::new(u64::from(placement.arena_index_bytes().get()))
                .ok_or_else(pressure)?,
        )
        .map_err(|denial| RecordAppendError::Denied(RecordAppendDenial::from_residency(denial)))?;
    let maximum_ranges = charge.bytes().saturating_sub(1024) as usize / 4096;
    if maximum_ranges < 3 {
        return Err(pressure());
    }
    if header.arena_capacity() != placement.arena_capacity().get() {
        return Err(RecordAppendError::Denied(
            RecordAppendDenial::PlacementFormatMismatch,
        ));
    }
    let alignment = header.arena_alignment();
    let mut owner = ExtentArenaAllocationOwner::new_tiered(
        placement.arena_capacity(),
        alignment,
        maximum_ranges,
        header.next_arena(),
        header.tier_epoch_start(),
    )
    .map_err(|_| damaged())?;
    owner.retain_capacity_charge(charge);
    let reader = FreeSpaceReader::serving(residency, format, access, header);
    let mut counters = ManifestDiscoveryCounterSnapshot::default();
    let mut restore_denial = None;
    reader
        .visit_arena_ranges(allocation, &mut counters, |entry| {
            if let Some(range) = entry.arena_free_range() {
                owner.restore_free_range(range).map_err(|denial| {
                    restore_denial = Some(denial);
                })?;
            }
            Ok(())
        })
        .map_err(|failure| match restore_denial {
            Some(cause @ super::ArenaAllocationDenial::RangeBudget { .. }) => {
                RecordAppendError::Denied(RecordAppendDenial::ArenaAllocationUnavailable(cause))
            }
            Some(_) => damaged(),
            None => super::super::planning::inline_plan_failure::manifest_lookup_failure(failure),
        })?;
    Ok(Arc::new(Mutex::new(owner)))
}

fn damaged() -> RecordAppendError {
    RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged)
}

fn pressure() -> RecordAppendError {
    RecordAppendError::Denied(RecordAppendDenial::PhysicalPressure)
}
