//! Root-only successor for one terminal head retired. Routes, segments and
//! the free-space tree are carried over unchanged: only the release-head root
//! and its block frontier move, and the free-space header is restamped for
//! the successor generation.

use worth_store_physical_format::{
    durable_artifact_checksum, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    PersistedTerminalReleaseHeadRetirementV1, RecordArtifactFile,
};

use super::{damaged, projection::ProjectedSuccessorRoot, RootRebaseContext};
use crate::physical_runtime::record_serving::{
    access::manifest_routing::ManifestDiscoveryCounterSnapshot,
    planning::prepared_payload::PreparedRecordPayloadPlan, RecordAppendError,
};

pub(super) fn project(
    context: &RootRebaseContext<'_>,
    prepared: &PreparedRecordPayloadPlan,
    retirement: &PersistedTerminalReleaseHeadRetirementV1,
    generation: u64,
) -> Result<ProjectedSuccessorRoot, RecordAppendError> {
    let current = context.current_root;
    let free = context.current_free_space;
    // A retirement shares its root step with no record effect: a group that
    // merged one in is denied rather than published around the head removal.
    if !prepared.is_record_less()
        || retirement.source_root_generation() != current.generation()
        || Some(retirement.source_root()) != current.release_custody_head_root()
        || retirement.source_next_block() != current.next_release_custody_head_block()
        || retirement.tree_identity() != current.tree_identity()
    {
        return Err(damaged());
    }
    let successor_free = DurableFreeSpaceManifestHeader::new_with_tier_epoch(
        generation,
        free.tree_identity(),
        free.node_capacity(),
        free.segment_page_capacity(),
        free.entry_count(),
        free.next_segment(),
        free.next_page(),
        free.next_extent(),
        free.next_arena(),
        free.tier_epoch_start(),
        free.arena_capacity(),
        free.arena_alignment(),
        free.next_block(),
        free.root(),
    )
    .ok_or_else(damaged)?;
    let free_bytes = successor_free.encode(context.format.declaration());
    let root = DurablePhysicalRootManifest::builder(
        generation,
        current.tree_identity(),
        current.node_capacity(),
        durable_artifact_checksum(&free_bytes),
    )
    .record_count(current.record_count())
    .next_block(current.next_block())
    .next_segment_block(current.next_segment_block())
    .next_release_custody_head_block(retirement.result_next_block())
    .routing_root(current.routing_root())
    .release_custody_head_root(retirement.result_root())
    .segment_root(current.segment_root())
    .free_space_root(successor_free.root())
    .tier_epoch_anchor(current.tier_epoch_anchor())
    .latest_blob_publication(current.latest_blob_publication())
    .latest_blob_quarantine(current.latest_blob_quarantine())
    .derived_family_directory(current.derived_family_directory())
    .last_inline_record(current.last_inline_record())
    .last_inline_segment(current.last_inline_segment())
    .admit()
    .ok_or_else(damaged)?;
    let mut manifests = vec![(
        RecordArtifactFile::FreeSpaceManifest { generation },
        free_bytes,
    )];
    manifests.extend(retirement.node_writes().iter().map(|write| {
        let reference = write.reference();
        (
            RecordArtifactFile::ReleaseCustodyHeadBlock {
                generation: reference.generation(),
                block: reference.block(),
            },
            write.frame().to_vec(),
        )
    }));
    Ok(ProjectedSuccessorRoot {
        free_space: successor_free,
        manifests,
        root,
        discoveries: [
            ManifestDiscoveryCounterSnapshot::default(),
            ManifestDiscoveryCounterSnapshot::default(),
            ManifestDiscoveryCounterSnapshot::default(),
        ],
    })
}
