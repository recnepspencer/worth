use std::collections::BTreeMap;

use worth_store_physical_format::{
    durable_artifact_checksum, CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader,
    DurablePhysicalRootManifest, PersistedRecordIdentity, RecordArtifactFile,
    RecordSegmentPageManifestEntry, SegmentGenerationCell, SegmentPageKey,
};

use super::{damaged, RootRebaseContext};
use crate::physical_runtime::record_serving::{
    access::{
        manifest_routing::{
            plan_manifest_updates, ManifestDiscoveryCounterSnapshot, ManifestReader,
            RootManifestUpdateRequest,
        },
        segment_membership::{
            plan_segment_membership_updates, SegmentMembershipPublicationPlan,
            SegmentMembershipUpdateContext,
        },
    },
    planning::{
        free_space_projection::{project_successor_free_space, FreeSpaceProjectionContext},
        free_space_routing::FreeSpacePublicationPlan,
        prepared_payload::PreparedRecordPayloadPlan,
    },
    publication::append_observation::PublicationObservation,
    RecordAppendError,
};

pub(super) struct ProjectedSuccessorRoot {
    pub(super) free_space: DurableFreeSpaceManifestHeader,
    pub(super) manifests: Vec<(RecordArtifactFile, Vec<u8>)>,
    pub(super) root: DurablePhysicalRootManifest,
    discoveries: [ManifestDiscoveryCounterSnapshot; 3],
}

struct ProjectedRecordManifest {
    root: DurablePhysicalRootManifest,
    blocks: Vec<(RecordArtifactFile, Vec<u8>)>,
    discovery: ManifestDiscoveryCounterSnapshot,
}

struct RootManifestProjection<'projection> {
    generation: u64,
    free_space: &'projection DurableFreeSpaceManifestHeader,
    free_space_bytes: &'projection [u8],
    segment: &'projection SegmentMembershipPublicationPlan,
    placements: &'projection BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    last_inline_record: Option<PersistedRecordIdentity>,
    last_inline_segment: Option<SegmentGenerationCell>,
}

pub(super) fn project_successor_root(
    context: &RootRebaseContext<'_>,
    prepared: &PreparedRecordPayloadPlan,
    generation: u64,
) -> Result<ProjectedSuccessorRoot, RecordAppendError> {
    let free_space = project_free_space(context, prepared, generation)?;
    let FreeSpacePublicationPlan {
        header: free_space_header,
        blocks: free_space_blocks,
        discovery: free_space_discovery,
    } = free_space;
    let free_space_bytes = free_space_header.encode(context.format.declaration());
    let segment = project_segment_membership(context, &prepared.segment_updates, generation)?;
    let (last_inline_record, last_inline_segment) = successor_inline_tail(
        context.current_root,
        prepared.last_inline_record,
        prepared.last_inline_segment,
    );
    let routed = project_record_manifest(
        context,
        RootManifestProjection {
            generation,
            free_space: &free_space_header,
            free_space_bytes: &free_space_bytes,
            segment: &segment,
            placements: &prepared.placements,
            last_inline_record,
            last_inline_segment,
        },
    )?;
    let mut manifests = free_space_blocks;
    manifests.push((
        RecordArtifactFile::FreeSpaceManifest { generation },
        free_space_bytes,
    ));
    manifests.extend(segment.blocks);
    manifests.extend(routed.blocks);
    Ok(ProjectedSuccessorRoot {
        free_space: free_space_header,
        manifests,
        root: routed.root,
        discoveries: [free_space_discovery, segment.discovery, routed.discovery],
    })
}

impl ProjectedSuccessorRoot {
    pub(super) fn observe_discovery(&self, observation: &mut PublicationObservation) {
        for discovery in self.discoveries {
            observation.manifest_blocks_read = observation
                .manifest_blocks_read
                .saturating_add(discovery.blocks_read());
            observation.manifest_comparisons = observation
                .manifest_comparisons
                .saturating_add(discovery.comparisons());
            observation.manifest_bytes_read = observation
                .manifest_bytes_read
                .saturating_add(discovery.bytes_read());
        }
    }
}

fn project_free_space(
    context: &RootRebaseContext<'_>,
    prepared: &PreparedRecordPayloadPlan,
    generation: u64,
) -> Result<FreeSpacePublicationPlan, RecordAppendError> {
    project_successor_free_space(
        FreeSpaceProjectionContext {
            allocation: context.allocation,
            residency: context.residency.clone(),
            format: context.format,
            access: context.access,
            current: context.current_free_space,
            successor_generation: generation,
            successor_capacity: context.placement.manifest_capacity().get(),
        },
        &prepared.inline_allocations,
        &prepared.placements,
    )
}

fn project_segment_membership(
    context: &RootRebaseContext<'_>,
    updates: &BTreeMap<SegmentPageKey, RecordSegmentPageManifestEntry>,
    generation: u64,
) -> Result<SegmentMembershipPublicationPlan, RecordAppendError> {
    plan_segment_membership_updates(
        SegmentMembershipUpdateContext {
            allocation: context.allocation,
            residency: context.residency.clone(),
            format: context.format,
            access: context.access,
            current: context.current_root,
            successor_generation: generation,
            successor_capacity: context.placement.manifest_capacity().get(),
        },
        updates,
    )
    .map_err(|_| damaged())
}

fn project_record_manifest(
    context: &RootRebaseContext<'_>,
    projection: RootManifestProjection<'_>,
) -> Result<ProjectedRecordManifest, RecordAppendError> {
    let reader = ManifestReader::serving(
        context.residency.clone(),
        context.format,
        context.access,
        context.current_root.clone(),
    );
    let projected = plan_manifest_updates(
        &reader,
        context.allocation,
        context.current_root,
        RootManifestUpdateRequest {
            successor_generation: projection.generation,
            successor_capacity: context.placement.manifest_capacity().get(),
            free_space_checksum: durable_artifact_checksum(projection.free_space_bytes),
            free_space_root: projection.free_space.root(),
            segment_root: projection.segment.root,
            next_segment_block: projection.segment.next_block,
            placements: projection.placements,
            last_inline_record: projection.last_inline_record,
            last_inline_segment: projection.last_inline_segment,
        },
    )
    .map_err(|_| damaged())?;
    Ok(ProjectedRecordManifest {
        root: projected.root,
        blocks: projected.blocks,
        discovery: projected.discovery,
    })
}

fn successor_inline_tail(
    current: &DurablePhysicalRootManifest,
    prepared_record: Option<PersistedRecordIdentity>,
    prepared_segment: Option<SegmentGenerationCell>,
) -> (
    Option<PersistedRecordIdentity>,
    Option<SegmentGenerationCell>,
) {
    let current_segment = current.last_inline_segment();
    let prepared_wins = match (prepared_segment, current_segment) {
        (Some(prepared), Some(current)) => {
            (prepared.segment_id().get(), prepared.generation().get())
                >= (current.segment_id().get(), current.generation().get())
        }
        (Some(_), None) => true,
        _ => false,
    };
    if prepared_wins {
        (prepared_record, prepared_segment)
    } else {
        (current.last_inline_record(), current_segment)
    }
}
