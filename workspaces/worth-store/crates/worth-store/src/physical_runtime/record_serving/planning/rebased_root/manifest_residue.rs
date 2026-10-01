//! Root-only successor for a proved orphan custody manifest. No payload,
//! segment-membership, or free-space state is changed by this plan.

use std::collections::{BTreeMap, BTreeSet};

use worth_store_physical_format::{
    durable_artifact_checksum, CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader,
    DurablePhysicalRootManifest, PersistedRecordIdentity, RecordArtifactFile,
};

use super::{
    assembly::{assemble_successor, RootAssemblyContext},
    damaged,
    projection::ProjectedSuccessorRoot,
    retirement::RetirementRootPlanningContext,
};
use crate::physical_runtime::record_serving::{
    access::manifest_routing::{
        plan_manifest_updates, ManifestDiscoveryCounterSnapshot, ManifestReader,
        RootManifestUpdateRequest,
    },
    publication::{append_observation::PublicationObservation, PublicationPlan},
    RecordAppendError,
};

pub(in crate::physical_runtime::record_serving) fn plan_manifest_residue_cleanup(
    context: RetirementRootPlanningContext<'_>,
    manifests: &[PersistedRecordIdentity],
    candidate: RecordArtifactFile,
) -> Result<
    (
        PublicationPlan,
        worth_store_physical_format::DurableFreeSpaceManifestHeader,
    ),
    RecordAppendError,
> {
    let current = context.current_root;
    let generation = current.generation().checked_add(1).ok_or_else(damaged)?;
    let RecordArtifactFile::CatalogCandidate { publication } = candidate else {
        return Err(damaged());
    };
    let reader = ManifestReader::serving(
        context.residency,
        context.format,
        context.access,
        current.clone(),
    );
    if manifests.is_empty() || manifests.len() > 2 {
        return Err(damaged());
    }
    let drops: BTreeSet<_> = manifests.iter().copied().collect();
    if drops.len() != manifests.len() {
        return Err(damaged());
    }
    let projected = plan_manifest_updates(
        &reader,
        context.allocation,
        current,
        RootManifestUpdateRequest {
            derived_updates: Default::default(),
            release_head_effect: None,
            successor_generation: generation,
            successor_capacity: current.node_capacity(),
            free_space_checksum: current.free_space_checksum(),
            free_space_root: current.free_space_root(),
            segment_root: current.segment_root(),
            next_segment_block: current.next_segment_block(),
            placements: &BTreeMap::<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>::new(),
            drops: &drops,
            last_inline_record: current.last_inline_record(),
            last_inline_segment: current.last_inline_segment(),
        },
    )
    .map_err(|_| damaged())?;
    // Every selected root generation has its own free-space header, even when
    // this root-only cleanup leaves the free-space tree and counters unchanged.
    let free = context.current_free;
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
        projected.root.tree_identity(),
        projected.root.node_capacity(),
        durable_artifact_checksum(&free_bytes),
    )
    .record_count(projected.root.record_count())
    .next_block(projected.root.next_block())
    .next_segment_block(projected.root.next_segment_block())
    .next_release_custody_head_block(projected.root.next_release_custody_head_block())
    .routing_root(projected.root.routing_root())
    .release_custody_head_root(projected.root.release_custody_head_root())
    .segment_root(projected.root.segment_root())
    .free_space_root(successor_free.root())
    .tier_epoch_anchor(projected.root.tier_epoch_anchor())
    .latest_blob_publication(projected.root.latest_blob_publication())
    .latest_blob_quarantine(projected.root.latest_blob_quarantine())
    .derived_family_directory(projected.root.derived_family_directory())
    .last_inline_record(projected.root.last_inline_record())
    .last_inline_segment(projected.root.last_inline_segment())
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
            media: context.media,
            format: context.format,
            current_root: current,
        },
        generation,
        ProjectedSuccessorRoot {
            free_space: successor_free,
            manifests: projected
                .blocks
                .into_iter()
                .chain([(
                    RecordArtifactFile::FreeSpaceManifest { generation },
                    free_bytes,
                )])
                .collect(),
            root,
            discoveries: [
                projected.discovery,
                ManifestDiscoveryCounterSnapshot::default(),
                ManifestDiscoveryCounterSnapshot::default(),
            ],
        },
    ))
}
