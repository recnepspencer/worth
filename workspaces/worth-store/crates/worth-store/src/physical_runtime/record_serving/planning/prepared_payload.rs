use std::collections::BTreeMap;

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PersistedRecordIdentity,
    RecordArtifactFile, RecordSegmentPageManifestEntry, SegmentGenerationCell, SegmentPageKey,
};

use super::super::{
    planning::{
        batch_placement::ClassifiedBatch,
        extent_placement::lower_extents,
        inline_segment_plan::{
            plan_inline_segments, InlineSegmentAllocation, InlineSegmentPlanningContext,
            WorkingSegment,
        },
        placement_context::PlacementPlanningContext,
    },
    publication::{
        append_observation::PublicationObservation, segment_publication::SegmentDataPlan,
        CandidateDataArtifact,
    },
    RecordAppendError,
};

pub(in crate::physical_runtime::record_serving) struct PreparedRecordPayloadPlan {
    pub(in crate::physical_runtime::record_serving) source_root: DurablePhysicalRootManifest,
    pub(in crate::physical_runtime::record_serving) manifest_capacity_transition:
        super::super::publication::PhysicalManifestCapacityTransition,
    pub(in crate::physical_runtime::record_serving) placement:
        super::super::AdmittedRecordPlacementPolicy,
    pub(in crate::physical_runtime::record_serving) records: Vec<PersistedRecordIdentity>,
    pub(in crate::physical_runtime::record_serving) data: Vec<CandidateDataArtifact>,
    pub(in crate::physical_runtime::record_serving) payload_manifests:
        Vec<(RecordArtifactFile, Vec<u8>)>,
    pub(in crate::physical_runtime::record_serving) placements:
        BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    pub(in crate::physical_runtime::record_serving) segment_updates:
        BTreeMap<SegmentPageKey, RecordSegmentPageManifestEntry>,
    pub(in crate::physical_runtime::record_serving) inline_allocations:
        Vec<InlineSegmentAllocation>,
    pub(in crate::physical_runtime::record_serving) last_inline_record:
        Option<PersistedRecordIdentity>,
    pub(in crate::physical_runtime::record_serving) last_inline_segment:
        Option<SegmentGenerationCell>,
    pub(in crate::physical_runtime::record_serving) observation: PublicationObservation,
    pub(in crate::physical_runtime::record_serving) requires_maintenance_protocol: bool,
}

pub(in crate::physical_runtime::record_serving) fn prepare_payload_plan(
    context: PlacementPlanningContext<'_>,
    classified: ClassifiedBatch,
    allow_published_reuse: bool,
) -> Result<PreparedRecordPayloadPlan, RecordAppendError> {
    let PlacementPlanningContext {
        allocation,
        media,
        format,
        access,
        current_root,
        current_free_space,
        frontier,
        placement,
        residency,
    } = context;
    let mut data = Vec::new();
    let mut payload_manifests = Vec::new();
    let mut placements = BTreeMap::new();
    let inline = plan_inline_segments(
        InlineSegmentPlanningContext {
            allocation,
            media,
            format,
            access,
            current_root,
            current_free_space,
            frontier,
            placement,
            placements: &mut placements,
            residency,
            allow_published_reuse,
        },
        classified.inline,
    )?;
    lower_extents(
        format,
        frontier,
        classified.extents,
        &mut data,
        &mut payload_manifests,
        &mut placements,
    )?;
    let (segment_updates, inline_allocations) = lower_segments(inline.segments, &mut data)?;
    let (last_inline_record, last_inline_segment) = last_inline_tail(&placements);
    let observation = PublicationObservation {
        records: classified.identities.len() as u64,
        logical_bytes: classified.logical_bytes,
        completed_bytes: 0,
        segment_artifacts: count_segments(&data),
        extent_artifacts: count_extents(&data),
        transfer_count: 0,
        peak_transfer_width: inline.peak_read_width as u64,
        explicit_copy_count: inline.source_copy_count,
        copied_bytes: inline.source_copied_bytes,
        peak_scratch_bytes: 0,
        manifest_blocks_read: 0,
        manifest_comparisons: 0,
        manifest_bytes_read: 0,
    };
    Ok(PreparedRecordPayloadPlan {
        source_root: current_root.clone(),
        manifest_capacity_transition:
            super::super::publication::PhysicalManifestCapacityTransition::PreserveCurrent,
        placement,
        records: classified.identities,
        data,
        payload_manifests,
        placements,
        segment_updates,
        inline_allocations,
        last_inline_record,
        last_inline_segment,
        observation,
        requires_maintenance_protocol: current_root.requires_maintenance_protocol(),
    })
}

fn lower_segments(
    segments: Vec<WorkingSegment>,
    data: &mut Vec<CandidateDataArtifact>,
) -> Result<
    (
        BTreeMap<SegmentPageKey, RecordSegmentPageManifestEntry>,
        Vec<InlineSegmentAllocation>,
    ),
    RecordAppendError,
> {
    let mut updates = BTreeMap::new();
    let mut allocations = Vec::with_capacity(segments.len());
    for segment in segments {
        allocations.push(segment.allocation());
        for entry in segment.membership_updates.iter().copied() {
            updates.insert(SegmentPageKey::from(entry), entry);
        }
        data.push(CandidateDataArtifact::Segment(SegmentDataPlan {
            artifact: RecordArtifactFile::Segment {
                segment: segment.segment.segment_id().get(),
                generation: segment.segment.generation().get(),
            },
            pages: segment.data_pages,
        }));
    }
    Ok((updates, allocations))
}

fn last_inline_tail(
    placements: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
) -> (
    Option<PersistedRecordIdentity>,
    Option<SegmentGenerationCell>,
) {
    placements
        .values()
        .filter_map(|entry| match entry {
            CurrentPhysicalRecordPlacement::Inline(value) => Some((
                value.segment().get(),
                value.page().get(),
                value.slot().get(),
                value.record(),
                value.segment_cell(),
            )),
            CurrentPhysicalRecordPlacement::Extent(_) => None,
        })
        .max_by_key(|value| (value.0, value.1, value.2))
        .map(|value| (Some(value.3), Some(value.4)))
        .unwrap_or((None, None))
}

fn count_segments(data: &[CandidateDataArtifact]) -> u64 {
    data.iter()
        .filter(|value| matches!(value, CandidateDataArtifact::Segment(_)))
        .count() as u64
}

fn count_extents(data: &[CandidateDataArtifact]) -> u64 {
    data.iter()
        .filter(|value| matches!(value, CandidateDataArtifact::Extent(_)))
        .count() as u64
}
