//! Exact routed V3 triple and complete selected control topology.

use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobRecordV1, CurrentPhysicalRecordPlacement, PersistedRecordIdentity,
    PhysicalInventoryTranscriptBuilderV1, PhysicalInventoryTranscriptV1,
    PhysicalRecordFormatDeclaration,
};
use worth_store_recovery_physics::VerifiedPendingWalReleaseCustody;

use super::super::{
    control_frames::{
        self, ObservedSelectedControls, SelectedArtifactSlice, SelectedControlMediaFingerprint,
    },
    release_heads, root_checkpoint, tier, SelectedMediaRejoinDenial as Denial,
    MAX_CONTROL_FRAME_BYTES, MAX_DISCOVERY_BYTES,
};
use super::{lineage, selection::Selection, topology};
use crate::physical_runtime::PhysicalRecoveryAllocationAdmission;

const MAX_TRANSCRIPT_ENTRIES: u64 =
    MAX_DISCOVERY_BYTES / (4 * std::mem::size_of::<CurrentPhysicalRecordPlacement>() as u64);

#[path = "controls/addressed.rs"]
mod addressed;
#[path = "controls/checkpoint_head_tree.rs"]
mod checkpoint_head_tree;
#[path = "controls/fingerprint.rs"]
mod fingerprint;
#[path = "controls/memory.rs"]
mod memory;
#[path = "controls/read.rs"]
mod read;

pub(super) struct Controls {
    routes: tier::routes::NoReleaseControlProvenance,
    source_routes: tier::routes::NoReleaseControlProvenance,
    checkpoint_routes: Option<tier::routes::NoReleaseControlProvenance>,
    selected_base: Option<ObservedSelectedControls>,
    selected_head_v2_controls: Option<release_heads::ObservedReleaseHeadControls>,
    source_topology: PhysicalInventoryTranscriptV1,
    published_topology: PhysicalInventoryTranscriptV1,
    descriptor: Vec<u8>,
    reservation: Vec<u8>,
    manifest: Vec<u8>,
    slices: Vec<SelectedArtifactSlice>,
    checkpoint_slices: Vec<SelectedArtifactSlice>,
}

impl Controls {
    pub(super) fn selected_base_attempt(
        &self,
        descriptor: PersistedRecordIdentity,
    ) -> Option<[u8; 16]> {
        self.selected_base
            .as_ref()?
            .attempt_for_descriptor(descriptor)
    }
    pub(super) fn selected_base(&self) -> Option<&ObservedSelectedControls> {
        self.selected_base.as_ref()
    }
    pub(super) fn selected_head_v2_controls(
        &self,
    ) -> Option<&release_heads::ObservedReleaseHeadControls> {
        self.selected_head_v2_controls.as_ref()
    }
    pub(super) fn pending_controls(&self) -> (&[u8], &[u8], &[u8]) {
        (&self.descriptor, &self.reservation, &self.manifest)
    }
    pub(super) fn manifest_bytes(&self) -> &[u8] {
        &self.manifest
    }
}

pub(super) fn observe(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selected: &Selection,
    format: PhysicalRecordFormatDeclaration,
    claim: &VerifiedPendingWalReleaseCustody,
    recovery_allocation: PhysicalRecoveryAllocationAdmission,
    checkpoint: &worth_store_physical_integrity::VerifiedCheckpointStream,
) -> Result<Controls, Denial> {
    let (expected_source, expected_published) = claim.topologies().ok_or(Denial::RoutingFrame)?;
    let mut source_builder = PhysicalInventoryTranscriptBuilderV1::new(
        &selected.source_root,
        &selected.source_free,
        format,
        MAX_TRANSCRIPT_ENTRIES,
    )
    .map_err(|_| Denial::RoutingFrame)?;
    let source_routes = tier::routes::verify_transcribed(
        discovery,
        &selected.source_root,
        &selected.source_free,
        format,
        &mut source_builder,
    )?;
    let source_partition =
        tier::released_partition::partition(discovery, source_routes.selected_routes(), format)?;
    let pending = [
        claim.descriptor_record(),
        claim.reservation_record(),
        claim.manifest_record(),
    ];
    if pending[0] == pending[1] || pending[0] == pending[2] || pending[1] == pending[2] {
        return Err(Denial::CertificateRoster);
    }
    // Manifest and reservation can already be selected before the descriptor
    // WAL member is redone. The selected checkpoint covers only the old set.
    let mut old_released = source_partition.released().clone();
    for record in &pending {
        old_released.remove(record);
    }
    // Historical ordered controls belong to postcheckpoint edges, not the
    // selected checkpoint's Batch catalog. Every one is re-read at its own
    // addressed candidate root by `ordered_walk`; only checkpoint-certified
    // controls may remain in `old_released` for the base join below.
    if claim.ordered_history().is_some() {
        let batches = claim.ordered_released_batches();
        if batches.is_empty()
            || batches.len() as u64
                >= worth_store_physical_format::MAX_CHECKPOINT_CERTIFICATE_RECORDS
        {
            return Err(Denial::CertificateRoster);
        }
        let mut admitted = BTreeSet::new();
        for batch in batches {
            for record in [
                batch.descriptor_frame().record(),
                batch.reservation_frame().record(),
                batch.manifest_frame().record(),
            ] {
                if pending.contains(&record) || !admitted.insert(record) {
                    return Err(Denial::CertificateRoster);
                }
                old_released.remove(&record);
            }
        }
    }
    let mut base_failed_ingest_slices = Vec::new();
    let mut checkpoint_routes = None;
    let mut checkpoint_slices = Vec::new();
    let mut selected_head_v2_controls = None;
    let selected_base = if let Some(base) = claim.selected_release() {
        let addressed = root_checkpoint::addressed::observe_release_base(
            discovery, selected, base, format, checkpoint,
        )?;
        base_failed_ingest_slices.extend_from_slice(source_partition.reservation_slices());
        tier::verify_failed_ingest_subset(
            discovery,
            source_partition.failed_ingest(),
            format,
            selected.source_root.generation(),
            base.checkpoint().source().identity().sequence().get(),
            &mut base_failed_ingest_slices,
        )?;
        Some(control_frames::observe(
            discovery,
            &addressed,
            base,
            &old_released,
            None,
            MAX_CONTROL_FRAME_BYTES,
        )?)
    } else if let Some(base) = claim.addressed_release_base() {
        let (observed, routes, slices) =
            addressed::observe(discovery, selected, format, base, claim, &old_released)?;
        checkpoint_routes = Some(routes);
        checkpoint_slices = slices;
        Some(observed)
    } else if let Some(base) = claim.selected_head_v2() {
        // The pending edge leaves the root this base selected.
        if base.selected_root() != &selected.source_root {
            return Err(Denial::CertificateRoster);
        }
        // Store's own proof that this root's head tree is the checkpoint head
        // tree, held apart from the roster physics minted: the two root
        // frames Store read carry one tree, or `ordered_walk` replays every
        // edge between them.
        checkpoint_head_tree::require_unmoved_without_history(
            &selected.checkpoint_source_root,
            &selected.source_root,
            claim.ordered_history().is_some(),
        )?;
        // No edge above the checkpoint unroutes a head or Batch control: a
        // released edge unroutes exactly its manifest's graph drops
        // (`ordered_released::verify_delta`), so these routes hold the closure.
        selected_head_v2_controls = Some(release_heads::observe_controls(
            discovery,
            format,
            source_routes.selected_routes(),
            selected.source_free.tier_epoch_start(),
            base,
            recovery_allocation.byte_limit(),
        )?);
        None
    } else {
        if let Some(history) = claim.ordered_history() {
            if history.selected_topology() != expected_source || !old_released.is_empty() {
                return Err(Denial::CertificateRoster);
            }
        } else {
            match claim.historical_batches() {
                [] if old_released.is_empty() => {}
                [historical] => {
                    let retained = [
                        historical.descriptor_record(),
                        historical.reservation_record(),
                        historical.manifest_record(),
                    ];
                    if retained[0] == retained[1]
                        || retained[0] == retained[2]
                        || retained[1] == retained[2]
                        || old_released.len() != retained.len()
                        || retained
                            .iter()
                            .any(|record| !old_released.contains_key(record))
                    {
                        return Err(Denial::CertificateRoster);
                    }
                }
                _ => return Err(Denial::CertificateRoster),
            }
        }
        None
    };
    let mut slices = topology::observe_memberships(
        discovery,
        &selected.source_root,
        &selected.source_free,
        format,
        &mut source_builder,
    )?;
    slices.extend(base_failed_ingest_slices);
    let source_topology = source_builder.finish().map_err(|_| Denial::RoutingFrame)?;
    topology::require_transcript(expected_source, source_topology)?;
    let mut published_builder = PhysicalInventoryTranscriptBuilderV1::new(
        &selected.root,
        &selected.free,
        format,
        MAX_TRANSCRIPT_ENTRIES,
    )
    .map_err(|_| Denial::RoutingFrame)?;
    let routes = tier::routes::verify_transcribed(
        discovery,
        &selected.root,
        &selected.free,
        format,
        &mut published_builder,
    )?;
    slices.extend(topology::observe_memberships(
        discovery,
        &selected.root,
        &selected.free,
        format,
        &mut published_builder,
    )?);
    let published_topology = published_builder
        .finish()
        .map_err(|_| Denial::RoutingFrame)?;
    topology::require_transcript(expected_published, published_topology)?;
    let partition =
        tier::released_partition::partition(discovery, routes.selected_routes(), format)?;
    let source_released = source_partition.released();
    let newly_published = pending
        .iter()
        .filter(|record| !source_released.contains_key(record))
        .count();
    if source_released.len().checked_add(newly_published) != Some(partition.released().len())
        || source_released
            .iter()
            .any(|(record, route)| partition.released().get(record) != Some(route))
        || pending
            .iter()
            .any(|record| !partition.released().contains_key(record))
    {
        return Err(Denial::CertificateRoster);
    }
    slices.extend_from_slice(partition.reservation_slices());
    tier::verify_failed_ingest_subset(
        discovery,
        partition.failed_ingest(),
        format,
        selected.root.generation(),
        claim.checkpoint().source().identity().sequence().get(),
        &mut slices,
    )?;
    let descriptor = read::frame(
        discovery,
        partition.released(),
        format,
        claim.descriptor_record(),
        &mut slices,
    )?;
    let reservation = read::frame(
        discovery,
        partition.released(),
        format,
        claim.reservation_record(),
        &mut slices,
    )?;
    let manifest = read::frame(
        discovery,
        partition.released(),
        format,
        claim.manifest_record(),
        &mut slices,
    )?;
    let BlobRecordV1::ReclaimDescriptorV3(drop) =
        decode_blob_record(&descriptor).map_err(|_| Denial::ControlFrame)?
    else {
        return Err(Denial::ControlFrame);
    };
    let BlobRecordV1::OriginalDropReserved(reserved) =
        decode_blob_record(&reservation).map_err(|_| Denial::ControlFrame)?
    else {
        return Err(Denial::ControlFrame);
    };
    let BlobRecordV1::DropSetManifestV3(dropped) =
        decode_blob_record(&manifest).map_err(|_| Denial::ControlFrame)?
    else {
        return Err(Denial::ControlFrame);
    };
    let base = drop.base();
    if drop != claim.descriptor()
        || drop.encode() != descriptor
        || reserved.encode() != reservation
        || dropped.encode() != manifest
        || <[u8; 32]>::from(Sha256::digest(&descriptor)) != claim.descriptor_frame_sha256()
        || <[u8; 32]>::from(Sha256::digest(&reservation)) != claim.reservation_frame_sha256()
        || <[u8; 32]>::from(Sha256::digest(&manifest)) != claim.manifest_frame_sha256()
        || base.manifest_record() != claim.manifest_record()
        || base.manifest_frame_sha256() != claim.manifest_frame_sha256()
        || base.store() != discovery.store_identity().bytes()
        || base.source_root_generation() != claim.source_root().generation()
        || base.candidate_root_generation() != selected.root.generation()
        || drop.custody().source_root_frame_sha256() != claim.source_root_sha256()
        || reserved.store() != base.store()
        || reserved.manifest_record() != claim.manifest_record()
        || reserved.manifest_frame_sha256() != claim.manifest_frame_sha256()
        || reserved.reclaim_attempt() != base.reclaim_attempt()
        || reserved.source_basis_digest() != base.source_basis_digest()
        || reserved.reserved_selected_generation() != base.source_root_generation()
        || reserved.request() != drop.custody().request()
        || dropped.store() != base.store()
        || dropped.reclaim_attempt() != base.reclaim_attempt()
        || dropped.source_basis_digest() != base.source_basis_digest()
        || dropped.source_basis().digest(base.store()) != base.source_basis_digest()
        || dropped.count() != base.manifest_count()
        || dropped
            .dropped()
            .iter()
            .any(|record| routes.selected_routes().contains_key(record))
        || !lineage::predecessor_matches(claim, selected_base.as_ref(), drop, &dropped)
    {
        return Err(Denial::ControlFrame);
    }
    read::fingerprint_selection(&mut slices, selected, claim)?;
    Ok(Controls {
        routes,
        source_routes,
        checkpoint_routes,
        selected_base,
        selected_head_v2_controls,
        source_topology,
        published_topology,
        descriptor,
        reservation,
        manifest,
        slices,
        checkpoint_slices,
    })
}
