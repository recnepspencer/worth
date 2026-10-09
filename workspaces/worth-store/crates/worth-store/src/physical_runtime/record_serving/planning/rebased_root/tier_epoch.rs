//! Root-only transition from legacy Primary arenas to a deterministic tier
//! namespace. No record route, free-space entry, or payload is rewritten.

use worth_store_physical_format::{
    durable_artifact_checksum, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    RecordArtifactFile,
};

use super::{
    assembly::{assemble_successor, RootAssemblyContext},
    damaged,
    projection::ProjectedSuccessorRoot,
};
use crate::physical_runtime::record_serving::{
    access::manifest_routing::ManifestDiscoveryCounterSnapshot,
    publication::{append_observation::PublicationObservation, PublicationPlan},
    AdmittedPhysicalRecordFormat, RecordAppendError,
};

pub(in crate::physical_runtime::record_serving) fn plan_tier_epoch_activation(
    current: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
    format: AdmittedPhysicalRecordFormat,
    media: &worth_store_physical_backend::QualifiedFilesystemMedia,
    candidate: RecordArtifactFile,
    epoch: u64,
    anchor: [u8; 32],
) -> Result<(PublicationPlan, DurableFreeSpaceManifestHeader), RecordAppendError> {
    let RecordArtifactFile::CatalogCandidate { publication } = candidate else {
        return Err(damaged());
    };
    if free.tier_epoch_start().is_some()
        || current.tier_epoch_anchor().is_some()
        || epoch != free.next_arena()
        || anchor == [0; 32]
    {
        return Err(damaged());
    }
    let generation = current.generation().checked_add(1).ok_or_else(damaged)?;
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
        Some(epoch),
        free.arena_capacity(),
        free.arena_alignment(),
        free.next_block(),
        free.root(),
    )
    .ok_or_else(damaged)?;
    let free_bytes = successor_free.encode(format.declaration());
    let root = DurablePhysicalRootManifest::builder(
        generation,
        current.tree_identity(),
        current.node_capacity(),
        durable_artifact_checksum(&free_bytes),
    )
    .record_count(current.record_count())
    .next_block(current.next_block())
    .next_segment_block(current.next_segment_block())
    .next_release_custody_head_block(current.next_release_custody_head_block())
    .routing_root(current.routing_root())
    .release_custody_head_root(current.release_custody_head_root())
    .segment_root(current.segment_root())
    .free_space_root(successor_free.root())
    .tier_epoch_anchor(Some(anchor))
    .latest_blob_publication(current.latest_blob_publication())
    .latest_blob_quarantine(current.latest_blob_quarantine())
    .derived_family_directory(current.derived_family_directory())
    .last_inline_record(current.last_inline_record())
    .last_inline_segment(current.last_inline_segment())
    .admit()
    .ok_or_else(damaged)?
    .with_maintenance_protocol();
    let plan = PublicationPlan {
        routing_metadata_bytes: None,
        arena_reservations: Vec::new(),
        generation,
        manifests: Vec::new(),
        root: RecordArtifactFile::RootManifest { generation },
        candidate,
        manifest: current.clone(),
        root_bytes: Vec::new(),
        previous_selector_candidate: RecordArtifactFile::RootSelectorCandidate {
            role: worth_store_physical_format::RootSelectorRole::Previous,
            publication,
        },
        previous_selector_bytes: Vec::new(),
        current_selector_candidate: RecordArtifactFile::RootSelectorCandidate {
            role: worth_store_physical_format::RootSelectorRole::Current,
            publication,
        },
        current_selector_bytes: Vec::new(),
        catalog_bytes: Vec::new(),
        observation: PublicationObservation::default(),
    };
    Ok(assemble_successor(
        plan,
        RootAssemblyContext {
            media,
            format,
            current_root: current,
        },
        generation,
        ProjectedSuccessorRoot {
            free_space: successor_free,
            manifests: vec![(
                RecordArtifactFile::FreeSpaceManifest { generation },
                free_bytes,
            )],
            root,
            discoveries: [
                ManifestDiscoveryCounterSnapshot::default(),
                ManifestDiscoveryCounterSnapshot::default(),
                ManifestDiscoveryCounterSnapshot::default(),
            ],
        },
    ))
}
