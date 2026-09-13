use std::collections::{BTreeMap, VecDeque};

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableInlineRecordPlacement, PageGenerationCell,
    PersistedRecordIdentity, PhysicalGenerationAuthority, PhysicalRecordSlot,
    RecordSegmentPageManifestEntry,
};

use super::{
    AdmittedPhysicalRecordFormat, AdmittedRecordPlacementPolicy, MaterializedInlineInput,
    PageDataPlan, PlannedPageMembership, PlanningSegment, RecordAllocationFrontier,
    RecordAppendDenial, RecordAppendError, WorkingSegment,
};
use crate::physical_runtime::record_serving::planning::{
    inline_page_packing::{fitting_prefix, new_page_fill_capacity, remaining_policy_capacity},
    inline_plan_failure::admitted_generation as generation,
    published_tail_page::{LoadedPublishedTailPage, PublishedTailPageGeometry},
};
use crate::physical_runtime::record_serving::work_semantics::integrity_admission::AdmittedCleanInlinePageRecord;

pub(super) fn append_to_last_page(
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
    segment: &mut PlanningSegment,
    loaded: LoadedPublishedTailPage,
    inline: &mut VecDeque<MaterializedInlineInput>,
    placements: &mut BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
) -> Result<(), RecordAppendError> {
    let LoadedPublishedTailPage {
        image,
        geometry,
        records: admitted_records,
    } = loaded;
    let count = fitting_record_count(format, placement, &geometry, inline);
    if count == 0 {
        return Ok(());
    }
    let page_generation = generation(geometry.generation().checked_add(1))?;
    let candidate_page = PhysicalGenerationAuthority::for_canonical_physical_format()
        .page_cell(segment.segment.segment_id(), geometry.page())
        .with_page_generation(page_generation);
    remap_existing_page_records(&admitted_records, segment, candidate_page, placements)?;
    let candidate_frame_index = segment.data_pages.len() as u32;
    segment.last_published_page = None;
    segment
        .candidate_pages
        .push(PlannedPageMembership::Candidate {
            page: candidate_page,
            frame_index: candidate_frame_index,
        });
    let records = take_inline_records(
        count,
        inline,
        segment,
        candidate_page,
        geometry.slot_count(),
        placements,
    )?;
    segment.data_pages.push(PageDataPlan {
        page: candidate_page,
        existing_frame: Some(image),
        records,
    });
    Ok(())
}

fn fitting_record_count(
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
    geometry: &PublishedTailPageGeometry,
    inline: &mut VecDeque<MaterializedInlineInput>,
) -> usize {
    fitting_prefix(
        inline.make_contiguous(),
        geometry.free_bytes() as usize,
        remaining_policy_capacity(format, placement, geometry.free_bytes() as usize),
    )
}

fn remap_existing_page_records(
    records: &[AdmittedCleanInlinePageRecord],
    segment: &PlanningSegment,
    candidate_page: PageGenerationCell,
    placements: &mut BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
) -> Result<(), RecordAppendError> {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    for descriptor in records {
        let slot = authority
            .slot_cell(
                segment.segment.segment_id(),
                candidate_page.page_id(),
                descriptor.slot,
            )
            .with_slot_generation(generation(Some(descriptor.slot_generation))?);
        let placement = DurableInlineRecordPlacement::new(
            descriptor.record,
            segment.segment,
            candidate_page,
            slot,
            segment.page_capacity,
            u64::from(descriptor.payload_bytes),
        )
        .ok_or(RecordAppendError::Denied(
            RecordAppendDenial::PublishedLayoutDamaged,
        ))?;
        placements.insert(
            descriptor.record,
            CurrentPhysicalRecordPlacement::Inline(placement),
        );
    }
    Ok(())
}

pub(super) fn append_new_page(
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
    frontier: &mut RecordAllocationFrontier,
    segment: &mut PlanningSegment,
    inline: &mut VecDeque<MaterializedInlineInput>,
    placements: &mut BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
) -> Result<(), RecordAppendError> {
    let free = new_page_fill_capacity(format, placement);
    let count = fitting_prefix(inline.make_contiguous(), free, free);
    if count == 0 {
        return Err(RecordAppendError::Denied(
            RecordAppendDenial::InlinePageFull,
        ));
    }
    let page_id = frontier.allocate_page().ok_or(RecordAppendError::Denied(
        RecordAppendDenial::PhysicalIdentityExhausted,
    ))?;
    let page = PhysicalGenerationAuthority::for_canonical_physical_format()
        .page_cell(segment.segment.segment_id(), page_id)
        .with_page_generation(generation(Some(1))?);
    let frame_index = segment.data_pages.len() as u32;
    let records = take_inline_records(count, inline, segment, page, 0, placements)?;
    segment
        .candidate_pages
        .push(PlannedPageMembership::Candidate { page, frame_index });
    segment.used_pages = segment
        .used_pages
        .checked_add(1)
        .ok_or(RecordAppendError::Denied(
            RecordAppendDenial::PhysicalIdentityExhausted,
        ))?;
    segment.data_pages.push(PageDataPlan {
        page,
        existing_frame: None,
        records,
    });
    Ok(())
}

fn take_inline_records(
    count: usize,
    inline: &mut VecDeque<MaterializedInlineInput>,
    segment: &PlanningSegment,
    page: PageGenerationCell,
    old_count: u16,
    placements: &mut BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
) -> Result<
    Vec<(
        PersistedRecordIdentity,
        worth_store_physical_format::SlotGenerationCell,
        Vec<u8>,
    )>,
    RecordAppendError,
> {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let slot_generation = generation(Some(1))?;
    (0..count)
        .map(|index| {
            let input = inline.pop_front().expect("fitting prefix exists");
            let slot = PhysicalRecordSlot::from_raw(old_count + index as u16 + 1)
                .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::InlinePageFull))?;
            let slot_cell = authority
                .slot_cell(segment.segment.segment_id(), page.page_id(), slot)
                .with_slot_generation(slot_generation);
            placements.insert(
                input.record,
                CurrentPhysicalRecordPlacement::Inline(
                    DurableInlineRecordPlacement::new(
                        input.record,
                        segment.segment,
                        page,
                        slot_cell,
                        segment.page_capacity,
                        input.bytes.len() as u64,
                    )
                    .expect("planner coordinates are consistent"),
                ),
            );
            Ok((input.record, slot_cell, input.bytes))
        })
        .collect()
}

pub(super) fn new_segment(
    frontier: &mut RecordAllocationFrontier,
    placement: AdmittedRecordPlacementPolicy,
) -> Result<PlanningSegment, RecordAppendError> {
    let segment = frontier
        .allocate_segment()
        .ok_or(RecordAppendError::Denied(
            RecordAppendDenial::PhysicalIdentityExhausted,
        ))?;
    Ok(PlanningSegment {
        segment: PhysicalGenerationAuthority::for_canonical_physical_format()
            .segment_cell(segment)
            .with_segment_generation(generation(Some(1))?),
        page_capacity: placement.segment_pages().get(),
        used_pages: 0,
        last_published_page: None,
        candidate_pages: Vec::new(),
        data_pages: Vec::new(),
    })
}

pub(super) fn finish_segment(
    segment: PlanningSegment,
) -> Result<WorkingSegment, RecordAppendError> {
    let data_page_count = u32::try_from(segment.data_pages.len())
        .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged))?;
    let membership_updates = segment
        .candidate_pages
        .into_iter()
        .map(|membership| match membership {
            PlannedPageMembership::Candidate { page, frame_index } => {
                RecordSegmentPageManifestEntry::new(
                    page,
                    segment.segment,
                    data_page_count,
                    frame_index,
                )
            }
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(RecordAppendError::Denied(
            RecordAppendDenial::PublishedLayoutDamaged,
        ))?;
    Ok(WorkingSegment {
        segment: segment.segment,
        page_capacity: segment.page_capacity,
        used_pages: segment.used_pages,
        membership_updates,
        data_pages: segment.data_pages,
    })
}
