use std::collections::BTreeMap;

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader, FreeSpaceKey,
    PersistedRecordIdentity, RecordAllocationClass, RecordFreeSpaceManifestEntry,
};

use super::super::planning::free_space_routing::{
    plan_free_space_successor, FreeSpacePublicationPlan, FreeSpaceSuccessorRequest, FreeSpaceUpdate,
};
use super::super::{
    planning::inline_segment_plan::InlineSegmentAllocation, AdmittedPhysicalRecordFormat,
    AdmittedRecordAccessPolicy, RecordAppendDenial, RecordAppendError,
};

mod committed_frontier;
use committed_frontier::CommittedAllocationFrontier;

pub(in crate::physical_runtime::record_serving) struct FreeSpaceProjectionContext<'plan> {
    pub(in crate::physical_runtime::record_serving) allocation:
        &'plan worth_store_buffer_pool::OperationAllocationGrant,
    pub(in crate::physical_runtime::record_serving) residency:
        super::super::residency::PhysicalResidencyWorkPort,
    pub(in crate::physical_runtime::record_serving) format: AdmittedPhysicalRecordFormat,
    pub(in crate::physical_runtime::record_serving) access: AdmittedRecordAccessPolicy,
    pub(in crate::physical_runtime::record_serving) current: &'plan DurableFreeSpaceManifestHeader,
    pub(in crate::physical_runtime::record_serving) successor_generation: u64,
    pub(in crate::physical_runtime::record_serving) successor_capacity: u16,
}

pub(in crate::physical_runtime::record_serving) fn project_successor_free_space(
    context: FreeSpaceProjectionContext<'_>,
    touched_segments: &[InlineSegmentAllocation],
    placements: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
) -> Result<FreeSpacePublicationPlan, RecordAppendError> {
    let FreeSpaceProjectionContext {
        allocation,
        residency,
        format,
        access,
        current,
        successor_generation,
        successor_capacity,
    } = context;
    let frontier = CommittedAllocationFrontier::from_publication(
        current,
        touched_segments,
        placements.values().copied(),
    )
    .ok_or_else(damaged)?;
    let segment_page_capacity = touched_segments
        .first()
        .map_or(current.segment_page_capacity(), |segment| {
            segment.page_capacity()
        });
    if touched_segments
        .iter()
        .any(|segment| segment.page_capacity() != segment_page_capacity)
    {
        return Err(damaged());
    }
    let mut updates = BTreeMap::new();
    for segment in touched_segments {
        let key = FreeSpaceKey::new(
            RecordAllocationClass::InlinePage,
            segment.segment().segment_id().get(),
        )
        .ok_or_else(damaged)?;
        let update = if segment.used_pages() < segment.page_capacity() {
            FreeSpaceUpdate::Available(
                RecordFreeSpaceManifestEntry::new(
                    RecordAllocationClass::InlinePage,
                    segment.segment().segment_id().get(),
                    u64::from(segment.used_pages() + 1),
                    u64::from(segment.page_capacity() - segment.used_pages()),
                    segment.segment().generation().get(),
                )
                .ok_or_else(damaged)?,
            )
        } else {
            FreeSpaceUpdate::Exhausted
        };
        updates.insert(key, update);
    }
    let extent_key = FreeSpaceKey::new(RecordAllocationClass::Extent, 1).expect("stable owner");
    let extent_update = if frontier.next_extent < u64::MAX {
        FreeSpaceUpdate::Available(
            RecordFreeSpaceManifestEntry::new(
                RecordAllocationClass::Extent,
                1,
                frontier.next_extent,
                u64::MAX - frontier.next_extent,
                1,
            )
            .ok_or_else(damaged)?,
        )
    } else {
        FreeSpaceUpdate::Exhausted
    };
    updates.insert(extent_key, extent_update);
    plan_free_space_successor(
        allocation,
        residency,
        format,
        access,
        current,
        FreeSpaceSuccessorRequest {
            generation: successor_generation,
            node_capacity: successor_capacity,
            segment_page_capacity,
            next_segment: frontier.next_segment,
            next_page: frontier.next_page,
            next_extent: frontier.next_extent,
            updates,
        },
    )
    .map_err(|_| damaged())
}

fn damaged() -> RecordAppendError {
    RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged)
}
