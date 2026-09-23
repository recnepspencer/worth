use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurableInlineRecordPlacement, DurablePhysicalRootManifest,
    FreeSpaceKey, PhysicalGeneration, PhysicalGenerationAuthority, RecordAllocationClass,
};

use super::super::{
    planning::inline_segment_plan::PlanningSegment, AdmittedPhysicalRecordFormat,
    AdmittedRecordPlacementPolicy, RecordAppendDenial, RecordAppendError,
};

pub(in crate::physical_runtime::record_serving) struct ReusableSegmentContext<'plan> {
    pub(in crate::physical_runtime::record_serving) allocation:
        &'plan worth_store_buffer_pool::OperationAllocationGrant,
    pub(in crate::physical_runtime::record_serving) residency:
        super::super::residency::PhysicalResidencyWorkPort,
    pub(in crate::physical_runtime::record_serving) format: AdmittedPhysicalRecordFormat,
    pub(in crate::physical_runtime::record_serving) access:
        super::super::AdmittedRecordAccessPolicy,
    pub(in crate::physical_runtime::record_serving) current_root:
        &'plan DurablePhysicalRootManifest,
    pub(in crate::physical_runtime::record_serving) current_free_space:
        &'plan DurableFreeSpaceManifestHeader,
    pub(in crate::physical_runtime::record_serving) placement: AdmittedRecordPlacementPolicy,
}

pub(in crate::physical_runtime::record_serving) fn load_reusable_segment(
    context: ReusableSegmentContext<'_>,
    last: Option<DurableInlineRecordPlacement>,
) -> Result<(Option<PlanningSegment>, usize), RecordAppendError> {
    load_segment(context, last, false)
}

pub(in crate::physical_runtime::record_serving) fn load_published_segment(
    context: ReusableSegmentContext<'_>,
    last: Option<DurableInlineRecordPlacement>,
) -> Result<(Option<PlanningSegment>, usize), RecordAppendError> {
    load_segment(context, last, true)
}

fn load_segment(
    context: ReusableSegmentContext<'_>,
    last: Option<DurableInlineRecordPlacement>,
    accept_exhausted: bool,
) -> Result<(Option<PlanningSegment>, usize), RecordAppendError> {
    let ReusableSegmentContext {
        allocation,
        residency,
        format,
        access,
        current_root,
        current_free_space,
        placement,
    } = context;
    let Some(last) = last else {
        return Ok((None, 0));
    };
    let mut discovery =
        super::super::access::manifest_routing::ManifestDiscoveryCounterSnapshot::default();
    let page = super::super::access::segment_membership::SegmentMembershipReader::serving(
        residency.clone(),
        format,
        access,
        current_root.clone(),
    )
    .locate(allocation, last.segment(), last.page(), &mut discovery)
    .map_err(super::inline_plan_failure::manifest_lookup_failure)?
    .ok_or(RecordAppendError::Denied(
        RecordAppendDenial::PublishedLayoutDamaged,
    ))?;
    if page.page_cell() != last.page_cell() || page.data_segment_cell() != last.segment_cell() {
        return Err(RecordAppendError::Denied(
            RecordAppendDenial::PublishedLayoutDamaged,
        ));
    }
    let key = FreeSpaceKey::new(RecordAllocationClass::InlinePage, last.segment().get())
        .expect("published segment identity is nonzero");
    let mut free_discovery =
        super::super::access::manifest_routing::ManifestDiscoveryCounterSnapshot::default();
    let Some(free) = super::super::planning::free_space_routing::FreeSpaceReader::serving(
        residency,
        format,
        access,
        current_free_space,
    )
    .locate(allocation, key, &mut free_discovery)
    .map_err(super::inline_plan_failure::manifest_lookup_failure)?
    else {
        let page_capacity = placement.segment_pages().get();
        if !accept_exhausted || last.segment_page_capacity() != page_capacity {
            return Ok((None, free_discovery.bytes_read() as usize));
        }
        return published_segment(
            last,
            page,
            page_capacity,
            page_capacity,
            free_discovery.bytes_read(),
        );
    };
    let used_pages =
        placement
            .segment_pages()
            .get()
            .checked_sub(u32::try_from(free.unallocated_count()).map_err(|_| {
                RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged)
            })?)
            .ok_or(RecordAppendError::Denied(
                RecordAppendDenial::PublishedLayoutDamaged,
            ))?;
    if used_pages == 0 || last.segment_page_capacity() != placement.segment_pages().get() {
        return Ok((None, free_discovery.bytes_read() as usize));
    }
    published_segment(
        last,
        page,
        placement.segment_pages().get(),
        used_pages,
        free_discovery.bytes_read(),
    )
}

fn published_segment(
    last: DurableInlineRecordPlacement,
    page: worth_store_physical_format::RecordSegmentPageManifestEntry,
    page_capacity: u32,
    used_pages: u32,
    bytes_read: u64,
) -> Result<(Option<PlanningSegment>, usize), RecordAppendError> {
    let next_generation = last
        .segment_generation()
        .checked_add(1)
        .and_then(|value| PhysicalGeneration::from_raw(value).ok())
        .ok_or(RecordAppendError::Denied(
            RecordAppendDenial::PhysicalIdentityExhausted,
        ))?;
    Ok((
        Some(PlanningSegment {
            segment: PhysicalGenerationAuthority::for_canonical_physical_format()
                .segment_cell(last.segment())
                .with_segment_generation(next_generation),
            page_capacity,
            used_pages,
            last_published_page: Some(page),
            candidate_pages: Vec::new(),
            data_pages: Vec::new(),
        }),
        bytes_read as usize,
    ))
}
