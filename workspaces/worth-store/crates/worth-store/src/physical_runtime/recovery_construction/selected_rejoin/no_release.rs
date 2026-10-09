//! Same-media selected NoRelease rejoin for ordinary unanchored roots.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::{
    AdmittedRecoveryFilesystemMedia, ArtifactCeiling, BoundedRecoveryFilesystemDiscovery,
    FixedArtifact, PageAddress, ReadGrant, UnchargedRead,
};
use worth_store_physical_format::{
    checkpoint_stream_encoded_digest, maximum_current_root_entries, DurableFreeSpaceManifestHeader,
    DurablePhysicalRootManifest, DurableRootSelector, RootSelectorRole,
};
use worth_store_recovery_physics::VerifiedSelectedNoReleaseCustody;

use super::{
    claimed_checkpoint, tier, wal_inventory, SelectedMediaRejoinDenial as Denial,
    MAX_CLEANUP_SAMPLE_BYTES, MAX_DISCOVERY_BYTES, MAX_DISCOVERY_ENTRIES,
};
use crate::physical_runtime::{
    CompletedPhysicalRecoveryFreshReopen, IntegrityAdmittedRecoveryWalFrameView,
    PhysicalRecoveryCoordination, PhysicalRecoveryFreshnessPort,
};

struct Selection {
    selector: Vec<u8>,
    root_bytes: Vec<u8>,
    free_bytes: Vec<u8>,
    source_root_bytes: Vec<u8>,
    checkpoint_bytes: Vec<u8>,
    root: DurablePhysicalRootManifest,
    free: DurableFreeSpaceManifestHeader,
}

impl Selection {
    fn same_bytes(&self, other: &Self) -> bool {
        self.selector == other.selector
            && self.root_bytes == other.root_bytes
            && self.free_bytes == other.free_bytes
            && self.source_root_bytes == other.source_root_bytes
            && self.checkpoint_bytes == other.checkpoint_bytes
    }
}

pub(in crate::physical_runtime::recovery_construction) fn observe_claim(
    coordination: &PhysicalRecoveryCoordination,
    media: AdmittedRecoveryFilesystemMedia,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedSelectedNoReleaseCustody,
    pause_before_final_reread: impl FnOnce(),
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        super::SelectedWalMediaFingerprint,
        super::control_frames::SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    let checkpoint = coordination
        .require_selected_checkpoint(claim.checkpoint())
        .map_err(|_| Denial::CheckpointBinding)?;
    tier::no_release::verify_marker_claim(claim, false, checkpoint.stream())?;
    let mut first = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let selected = observe_selection(&mut first, reopen, claim)?;
    let controls = tier::routes::verify(
        &mut first,
        &selected.root,
        &selected.free,
        reopen.format(),
        Some(claim.checkpoint().source().identity().sequence().get()),
    )?;
    let selected_wal =
        wal_inventory::admit_complete_inventory(&mut first, coordination).map_err(|denial| {
            denial.at_resident_boundary(
                crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary::FirstWalAdmission,
            )
        })?;
    let media = first.finish();
    let sample = PhysicalRecoveryFreshnessPort::sample_binding(
        coordination,
        &media,
        claim.checkpoint(),
        IntegrityAdmittedRecoveryWalFrameView::from_frames(selected_wal.frames()),
        MAX_DISCOVERY_ENTRIES,
        wal_inventory::MAX_WAL_BYTES,
        MAX_CLEANUP_SAMPLE_BYTES,
    )
    .map_err(Denial::binding_sampling)?;
    tier::no_release::verify_selected_tail(
        &sample,
        &controls,
        reopen.format(),
        selected.root.generation(),
    )?;
    let selected_wal = selected_wal.into_fingerprint();
    pause_before_final_reread();
    let mut final_read = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let reread = observe_selection(&mut final_read, reopen, claim)?;
    if !selected.same_bytes(&reread) {
        return Err(Denial::RootBinding);
    }
    let reread_controls = tier::routes::verify(
        &mut final_read,
        &reread.root,
        &reread.free,
        reopen.format(),
        Some(claim.checkpoint().source().identity().sequence().get()),
    )?;
    if controls != reread_controls {
        return Err(Denial::ControlFrame);
    }
    let final_wal = wal_inventory::admit_complete_inventory(&mut final_read, coordination)
        .map_err(|denial| {
            denial.at_resident_boundary(
                crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary::FinalWalAdmission,
            )
        })?;
    if !selected_wal.matches_reread(&final_wal) {
        return Err(Denial::WalFate);
    }
    Ok((
        final_read.finish(),
        final_wal.into_fingerprint(),
        controls.into_media_fingerprint(),
    ))
}

fn observe_selection(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedSelectedNoReleaseCustody,
) -> Result<Selection, Denial> {
    let format = reopen.format();
    let selector_bytes = discovery
        .read(
            ArtifactCeiling::fixed(FixedArtifact::CurrentRootSelector),
            ReadGrant::ceiling_only(),
        )
        .observed()
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingSelector)?;
    let selector = DurableRootSelector::decode(&selector_bytes).map_err(|_| Denial::RootBinding)?;
    if selector.encode() != selector_bytes.as_slice()
        || selector.store_identity() != discovery.store_identity()
        || selector.format() != format
        || selector.role() != RootSelectorRole::Current
        || selector.root_generation() != claim.selected_root().generation()
        || selector_bytes != reopen.fresh_reopen_occurrence().selector().bytes()
    {
        return Err(Denial::RootBinding);
    }
    let root_bytes = discovery
        .read(
            ArtifactCeiling::page(
                format,
                PageAddress::RootManifest {
                    generation: selector.root_generation(),
                },
            ),
            ReadGrant::ceiling_only(),
        )
        .observed()
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingRoot)?;
    let (root, decoded_format) =
        DurablePhysicalRootManifest::decode(&root_bytes, claim.selected_root().node_capacity())
            .map_err(|_| Denial::RootBinding)?;
    if decoded_format != format
        || root != *claim.selected_root()
        || root != *reopen.root()
        || root.tier_epoch_anchor().is_some()
        || root.encode(format) != root_bytes
        || root_bytes != reopen.fresh_reopen_occurrence().root().bytes()
        || <[u8; 32]>::from(Sha256::digest(&root_bytes)) != claim.selected_root_sha256()
    {
        return Err(Denial::RootBinding);
    }
    let free_bytes = discovery
        .read(
            ArtifactCeiling::page(
                format,
                PageAddress::FreeSpaceManifest {
                    generation: root.generation(),
                },
            ),
            ReadGrant::ceiling_only(),
        )
        .observed()
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingFrame)?;
    let free = tier::selection::validate_selected_free_header(
        &free_bytes,
        discovery.store_identity(),
        format,
        &root,
    )?;
    if free.tier_epoch_start().is_some() || free.encode(format) != free_bytes {
        return Err(Denial::RootBinding);
    }
    let checkpoint_bytes = discovery
        .read(
            claimed_checkpoint(claim.checkpoint().encoded_bytes())?,
            ReadGrant::ceiling_only(),
        )
        .observed()
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingCheckpoint)?;
    if checkpoint_bytes.len() as u64 != claim.checkpoint().encoded_bytes()
        || checkpoint_stream_encoded_digest(&checkpoint_bytes)
            != claim.checkpoint().encoded_digest()
    {
        return Err(Denial::CheckpointBinding);
    }
    let source = claim.checkpoint().source().root();
    let source_root_bytes = discovery
        .read(
            ArtifactCeiling::page(
                format,
                PageAddress::RootManifest {
                    generation: source.generation(),
                },
            ),
            ReadGrant::ceiling_only(),
        )
        .observed()
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingRoot)?;
    let (source_root, source_format) = DurablePhysicalRootManifest::decode(
        &source_root_bytes,
        maximum_current_root_entries(format),
    )
    .map_err(|_| Denial::RootBinding)?;
    if source_format != format
        || source_root.generation() != source.generation()
        || source_root.tree_identity() != source.tree_identity()
        || source_root.encode(format) != source_root_bytes
        || <[u8; 32]>::from(Sha256::digest(&source_root_bytes))
            != claim.checkpoint_source_root_sha256()
    {
        return Err(Denial::RootBinding);
    }
    Ok(Selection {
        selector: selector_bytes,
        root_bytes,
        free_bytes,
        source_root_bytes,
        checkpoint_bytes,
        root,
        free,
    })
}
