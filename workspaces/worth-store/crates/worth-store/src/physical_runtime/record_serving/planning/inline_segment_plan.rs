use std::collections::{BTreeMap, VecDeque};

use worth_store_physical_backend::QualifiedFilesystemMedia;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PageGenerationCell,
    PersistedRecordIdentity, RecordSegmentPageManifestEntry, SegmentGenerationCell,
};

use super::super::{
    planning::batch_placement::{
        materialize_inline_inputs, MaterializedInlineInput, PendingInlineInput,
    },
    planning::published_segment_reuse::{load_reusable_segment, ReusableSegmentContext},
    planning::published_tail_page::load_published_tail_page,
    planning::reusable_inline_tail::last_inline_placement,
    publication::segment_publication::PageDataPlan,
    residency::{serving_artifacts::ServingRecordArtifacts, PhysicalResidencyWorkPort},
    AdmittedPhysicalRecordFormat, AdmittedRecordPlacementPolicy, RecordAllocationFrontier,
    RecordAppendDenial, RecordAppendError,
};

mod page_allocation;

use page_allocation::{append_new_page, append_to_last_page, finish_segment, new_segment};

pub(in crate::physical_runtime::record_serving) struct WorkingSegment {
    pub(in crate::physical_runtime::record_serving) segment: SegmentGenerationCell,
    pub(in crate::physical_runtime::record_serving) page_capacity: u32,
    pub(in crate::physical_runtime::record_serving) used_pages: u32,
    pub(in crate::physical_runtime::record_serving) membership_updates:
        Vec<RecordSegmentPageManifestEntry>,
    pub(in crate::physical_runtime::record_serving) data_pages: Vec<PageDataPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct InlineSegmentAllocation {
    segment: SegmentGenerationCell,
    page_capacity: u32,
    used_pages: u32,
}

impl WorkingSegment {
    pub(in crate::physical_runtime::record_serving) const fn allocation(
        &self,
    ) -> InlineSegmentAllocation {
        InlineSegmentAllocation {
            segment: self.segment,
            page_capacity: self.page_capacity,
            used_pages: self.used_pages,
        }
    }
}

impl InlineSegmentAllocation {
    pub(in crate::physical_runtime) const fn segment(self) -> SegmentGenerationCell {
        self.segment
    }

    pub(in crate::physical_runtime) const fn page_capacity(self) -> u32 {
        self.page_capacity
    }

    pub(in crate::physical_runtime) const fn used_pages(self) -> u32 {
        self.used_pages
    }
}

pub(in crate::physical_runtime::record_serving) struct PlannedInlineSegments {
    pub(in crate::physical_runtime::record_serving) segments: Vec<WorkingSegment>,
    pub(in crate::physical_runtime::record_serving) peak_read_width: usize,
    pub(in crate::physical_runtime::record_serving) source_copy_count: u64,
    pub(in crate::physical_runtime::record_serving) source_copied_bytes: u64,
}

pub(in crate::physical_runtime::record_serving) struct PlanningSegment {
    pub(in crate::physical_runtime::record_serving) segment: SegmentGenerationCell,
    pub(in crate::physical_runtime::record_serving) page_capacity: u32,
    pub(in crate::physical_runtime::record_serving) used_pages: u32,
    pub(in crate::physical_runtime::record_serving) last_published_page:
        Option<RecordSegmentPageManifestEntry>,
    pub(in crate::physical_runtime::record_serving) candidate_pages: Vec<PlannedPageMembership>,
    pub(in crate::physical_runtime::record_serving) data_pages: Vec<PageDataPlan>,
}

pub(in crate::physical_runtime::record_serving) enum PlannedPageMembership {
    Candidate {
        page: PageGenerationCell,
        frame_index: u32,
    },
}

pub(in crate::physical_runtime::record_serving) struct InlineSegmentPlanningContext<'plan> {
    pub(in crate::physical_runtime::record_serving) allocation:
        &'plan worth_store_buffer_pool::OperationAllocationGrant,
    pub(in crate::physical_runtime::record_serving) media: &'plan QualifiedFilesystemMedia,
    pub(in crate::physical_runtime::record_serving) format: AdmittedPhysicalRecordFormat,
    pub(in crate::physical_runtime::record_serving) access:
        super::super::AdmittedRecordAccessPolicy,
    pub(in crate::physical_runtime::record_serving) current_root:
        &'plan DurablePhysicalRootManifest,
    pub(in crate::physical_runtime::record_serving) current_free_space:
        &'plan worth_store_physical_format::DurableFreeSpaceManifestHeader,
    pub(in crate::physical_runtime::record_serving) frontier: &'plan mut RecordAllocationFrontier,
    pub(in crate::physical_runtime::record_serving) placement: AdmittedRecordPlacementPolicy,
    pub(in crate::physical_runtime::record_serving) placements:
        &'plan mut BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    pub(in crate::physical_runtime::record_serving) residency: PhysicalResidencyWorkPort,
    pub(in crate::physical_runtime::record_serving) allow_published_reuse: bool,
}

pub(in crate::physical_runtime::record_serving) fn plan_inline_segments(
    context: InlineSegmentPlanningContext<'_>,
    inline: Vec<PendingInlineInput>,
) -> Result<PlannedInlineSegments, RecordAppendError> {
    if inline.is_empty() {
        return Ok(PlannedInlineSegments {
            segments: Vec::new(),
            peak_read_width: 0,
            source_copy_count: 0,
            source_copied_bytes: 0,
        });
    }
    let InlineSegmentPlanningContext {
        allocation,
        media,
        format,
        access,
        current_root,
        current_free_space,
        frontier,
        placement,
        placements,
        residency,
        allow_published_reuse,
    } = context;
    let artifacts = ServingRecordArtifacts::serving(media, residency.clone());
    let last_inline = if allow_published_reuse {
        last_inline_placement(
            allocation,
            residency.clone(),
            format,
            access,
            current_root,
            placement,
        )?
    } else {
        None
    };
    let (mut active, mut peak_read_width) = if allow_published_reuse {
        load_reusable_segment(
            ReusableSegmentContext {
                allocation,
                residency,
                format,
                access,
                current_root,
                current_free_space,
                placement,
            },
            last_inline,
        )?
    } else {
        (None, 0)
    };
    let loaded_tail = if let (Some(last), Some(segment)) = (last_inline, active.as_ref()) {
        let loaded = load_published_tail_page(
            allocation,
            &artifacts,
            media.store_identity(),
            format,
            last,
            segment,
        )?;
        peak_read_width = peak_read_width.max(loaded.image.bytes().len());
        Some(loaded)
    } else {
        None
    };
    let materialized = materialize_inline_inputs(inline)?;
    let mut inline = VecDeque::from(materialized.records);
    let mut plans = Vec::new();
    if let (Some(loaded), Some(segment)) = (loaded_tail, active.as_mut()) {
        append_to_last_page(format, placement, segment, loaded, &mut inline, placements)?;
    }
    while !inline.is_empty() {
        if active
            .as_ref()
            .is_some_and(|segment| segment.used_pages == segment.page_capacity)
        {
            let full = active.take().unwrap();
            if !full.data_pages.is_empty() {
                plans.push(finish_segment(full)?);
            }
        }
        if active.is_none() {
            active = Some(new_segment(frontier, placement)?);
        }
        append_new_page(
            format,
            placement,
            frontier,
            active.as_mut().unwrap(),
            &mut inline,
            placements,
        )?;
    }
    if let Some(active) = active {
        if !active.data_pages.is_empty() {
            plans.push(finish_segment(active)?);
        }
    }
    Ok(PlannedInlineSegments {
        segments: plans,
        peak_read_width,
        source_copy_count: materialized.copy_count,
        source_copied_bytes: materialized.copied_bytes,
    })
}
