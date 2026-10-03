use std::collections::BTreeMap;

use worth_store_physical_format::{
    durable_artifact_checksum, CurrentPhysicalRecordPlacement, DerivedFamilyRootDirectoryBinding,
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, IndexedThroughBlobPublication,
    PersistedRecordIdentity, RecordArtifactFile, RecordSegmentPageManifestEntry,
    SegmentGenerationCell, SegmentPageKey,
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
        inline_plan_failure::manifest_lookup_failure,
        prepared_payload::PreparedRecordPayloadPlan,
    },
    publication::append_observation::PublicationObservation,
    RecordAppendError,
};

pub(super) struct ProjectedSuccessorRoot {
    pub(super) free_space: DurableFreeSpaceManifestHeader,
    pub(super) manifests: Vec<(RecordArtifactFile, Vec<u8>)>,
    pub(super) root: DurablePhysicalRootManifest,
    pub(super) discoveries: [ManifestDiscoveryCounterSnapshot; 3],
}

struct ProjectedRecordManifest {
    root: DurablePhysicalRootManifest,
    blocks: Vec<(RecordArtifactFile, Vec<u8>)>,
    discovery: ManifestDiscoveryCounterSnapshot,
}

struct RootManifestProjection<'projection> {
    derived_updates: crate::physical_runtime::record_serving::planning::prepared_root_projection::DerivedRootUpdates,
    release_head_effect:
        Option<&'projection worth_store_physical_format::PersistedReleaseCustodyHeadEffectV1>,
    generation: u64,
    free_space: &'projection DurableFreeSpaceManifestHeader,
    free_space_bytes: &'projection [u8],
    segment: &'projection SegmentMembershipPublicationPlan,
    placements: &'projection BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    drops: &'projection std::collections::BTreeSet<PersistedRecordIdentity>,
    last_inline_record: Option<PersistedRecordIdentity>,
    last_inline_segment: Option<SegmentGenerationCell>,
}

pub(super) fn project_successor_root(
    context: &RootRebaseContext<'_>,
    prepared: &PreparedRecordPayloadPlan,
    generation: u64,
) -> Result<ProjectedSuccessorRoot, RecordAppendError> {
    if prepared.blob_reuse_source_fence
        && context.current_root.generation() != prepared.source_root.generation()
    {
        return Err(damaged());
    }
    if let Some(expected_previous) = prepared.derived_updates.expected_previous_directory {
        let Some(directory) = prepared.derived_updates.directory else {
            return Err(damaged());
        };
        if !directory_rebase_matches(
            context.current_root.latest_blob_publication(),
            context.current_root.derived_family_directory(),
            directory.indexed_through_blob_publication(),
            expected_previous,
            prepared
                .derived_updates
                .indexed_through_quarantine
                .flatten(),
            context.current_root.latest_blob_quarantine(),
            prepared
                .derived_updates
                .released_directory_rebinding
                .is_some(),
            &prepared.drop_records,
        ) {
            return Err(damaged());
        }
    }
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
            derived_updates: prepared.derived_updates,
            release_head_effect: prepared.release_head_effect.as_ref(),
            generation,
            free_space: &free_space_header,
            free_space_bytes: &free_space_bytes,
            segment: &segment,
            placements: &prepared.placements,
            drops: &prepared.drop_records,
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
    if let Some(transition) = prepared.release_head_effect.as_ref() {
        manifests.extend(transition.node_writes().iter().map(|write| {
            let reference = write.reference();
            (
                RecordArtifactFile::ReleaseCustodyHeadBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                },
                write.frame().to_vec(),
            )
        }));
    }
    Ok(ProjectedSuccessorRoot {
        free_space: free_space_header,
        manifests,
        root: routed.root,
        discoveries: [free_space_discovery, segment.discovery, routed.discovery],
    })
}

fn directory_rebase_matches(
    current_blob_publication: Option<IndexedThroughBlobPublication>,
    current_directory: Option<DerivedFamilyRootDirectoryBinding>,
    proposed_indexed_through: Option<IndexedThroughBlobPublication>,
    expected_previous: Option<DerivedFamilyRootDirectoryBinding>,
    proposed_quarantine: Option<PersistedRecordIdentity>,
    current_quarantine: Option<PersistedRecordIdentity>,
    released_rebinding: bool,
    drops: &std::collections::BTreeSet<PersistedRecordIdentity>,
) -> bool {
    if current_directory != expected_previous {
        return false;
    }
    if released_rebinding {
        // A released rebinding only clears the watermark naming a dropped
        // publication; the directory may lag a newer surviving publication
        // and quarantine, which stay the root's own latest hints.
        return proposed_indexed_through.is_none()
            && expected_previous
                .and_then(|binding| binding.indexed_through_blob_publication())
                .is_some_and(|indexed| drops.contains(&indexed.record()));
    }
    current_blob_publication == proposed_indexed_through
        && current_quarantine == proposed_quarantine
}

#[cfg(test)]
mod directory_rebase_tests {
    use super::*;

    #[test]
    fn competing_directory_with_same_watermark_cannot_overwrite_selected_root() {
        let record = |ordinal| PersistedRecordIdentity::new([7; 16], ordinal).unwrap();
        let watermark = IndexedThroughBlobPublication::new(4, record(1), [8; 32]).unwrap();
        let originally_selected =
            DerivedFamilyRootDirectoryBinding::new(record(2), Some(watermark));
        let competing_selected = DerivedFamilyRootDirectoryBinding::new(record(3), Some(watermark));
        assert!(directory_rebase_matches(
            Some(watermark),
            Some(originally_selected),
            Some(watermark),
            Some(originally_selected),
            None,
            None,
            false,
            &std::collections::BTreeSet::new(),
        ));
        assert!(!directory_rebase_matches(
            Some(watermark),
            Some(competing_selected),
            Some(watermark),
            Some(originally_selected),
            None,
            None,
            false,
            &std::collections::BTreeSet::new(),
        ));
        assert!(!directory_rebase_matches(
            Some(watermark),
            Some(originally_selected),
            Some(watermark),
            Some(originally_selected),
            Some(record(4)),
            Some(record(5)),
            false,
            &std::collections::BTreeSet::new(),
        ));
    }
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
            arena_capacity: context.placement.arena_capacity(),
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
    .map_err(manifest_lookup_failure)
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
            derived_updates: projection.derived_updates,
            release_head_effect: projection.release_head_effect,
            successor_generation: projection.generation,
            successor_capacity: context.placement.manifest_capacity().get(),
            free_space_checksum: durable_artifact_checksum(projection.free_space_bytes),
            free_space_root: projection.free_space.root(),
            segment_root: projection.segment.root,
            next_segment_block: projection.segment.next_block,
            placements: projection.placements,
            drops: projection.drops,
            last_inline_record: projection.last_inline_record,
            last_inline_segment: projection.last_inline_segment,
        },
    )
    .map_err(manifest_lookup_failure)?;
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
