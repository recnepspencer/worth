//! Same-media NoRelease rejoin for ordinary unanchored roots, joined to the
//! selected checkpoint's marker or, before the first checkpoint, to the
//! generation-zero basis.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::{
    AdmittedRecoveryFilesystemMedia, ArtifactCeiling, BoundedRecoveryFilesystemDiscovery,
    FixedArtifact, PageAddress, ReadGrant, StreamArtifact, UnchargedRead,
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
    AbsentCheckpointWitness, CompletedPhysicalRecoveryFreshReopen,
    ConfiguredPhysicalDurabilityDeclaration, IntegrityAdmittedRecoveryWalFrameView,
    PhysicalRecoveryCoordination, PhysicalRecoveryFreshnessPort, StoreRecoverySamplingBasis,
};

/// What a NoRelease rejoin is joined to.
#[derive(Clone, Copy)]
pub(in crate::physical_runtime::recovery_construction) enum NoReleaseRejoinBasis<'claim> {
    /// The selected checkpoint's positive marker claim.
    Selected(&'claim VerifiedSelectedNoReleaseCustody),
    /// Before the first checkpoint: this coordination's absence witness, and
    /// every WAL binding carrying the declared durability policy. Sampling
    /// refuses a witness the coordination did not mint.
    GenerationZero(
        &'claim AbsentCheckpointWitness,
        ConfiguredPhysicalDurabilityDeclaration,
    ),
}

impl NoReleaseRejoinBasis<'_> {
    /// Checkpoint sequences are nonzero, so zero names the generation-zero
    /// basis, before which no checkpoint can have expired a session.
    fn checkpoint_sequence(self) -> u64 {
        match self {
            Self::Selected(claim) => claim.checkpoint().source().identity().sequence().get(),
            Self::GenerationZero(..) => 0,
        }
    }
}

impl<'claim> NoReleaseRejoinBasis<'claim> {
    fn sampling(self) -> StoreRecoverySamplingBasis<'claim> {
        match self {
            Self::Selected(claim) => StoreRecoverySamplingBasis::Checkpoint(claim.checkpoint()),
            Self::GenerationZero(absent, declaration) => {
                StoreRecoverySamplingBasis::GenerationZero(absent, declaration)
            }
        }
    }
}

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
    basis: NoReleaseRejoinBasis<'_>,
    pause_before_final_reread: impl FnOnce(),
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        super::SelectedWalMediaFingerprint,
        super::control_frames::SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    if let NoReleaseRejoinBasis::Selected(claim) = basis {
        let checkpoint = coordination
            .require_selected_checkpoint(claim.checkpoint())
            .map_err(|_| Denial::CheckpointBinding)?;
        tier::no_release::verify_marker_claim(claim, false, checkpoint.stream())?;
    }
    let mut first = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let selected = observe_selection(&mut first, reopen, basis)?;
    let controls = tier::routes::verify(
        &mut first,
        &selected.root,
        &selected.free,
        reopen.format(),
        Some(basis.checkpoint_sequence()),
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
        basis.sampling(),
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
    let reread = observe_selection(&mut final_read, reopen, basis)?;
    if !selected.same_bytes(&reread) {
        return Err(Denial::RootBinding);
    }
    let reread_controls = tier::routes::verify(
        &mut final_read,
        &reread.root,
        &reread.free,
        reopen.format(),
        Some(basis.checkpoint_sequence()),
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
    basis: NoReleaseRejoinBasis<'_>,
) -> Result<Selection, Denial> {
    let format = reopen.format();
    let expected_root = match basis {
        NoReleaseRejoinBasis::Selected(claim) => claim.selected_root(),
        NoReleaseRejoinBasis::GenerationZero(..) => reopen.root(),
    };
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
        || selector.root_generation() != expected_root.generation()
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
        DurablePhysicalRootManifest::decode(&root_bytes, expected_root.node_capacity())
            .map_err(|_| Denial::RootBinding)?;
    if decoded_format != format
        || root != *expected_root
        || root != *reopen.root()
        || root.tier_epoch_anchor().is_some()
        || root.encode(format) != root_bytes
        || root_bytes != reopen.fresh_reopen_occurrence().root().bytes()
        || matches!(basis, NoReleaseRejoinBasis::Selected(claim)
            if <[u8; 32]>::from(Sha256::digest(&root_bytes)) != claim.selected_root_sha256())
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
    let (checkpoint_bytes, source_root_bytes) = match basis {
        NoReleaseRejoinBasis::Selected(claim) => observe_checkpoint(discovery, format, claim)?,
        NoReleaseRejoinBasis::GenerationZero(..) => {
            observe_absent_checkpoint(discovery)?;
            (Vec::new(), Vec::new())
        }
    };
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

/// The selected checkpoint stream and its source root, both bound to the claim.
fn observe_checkpoint(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    claim: &VerifiedSelectedNoReleaseCustody,
) -> Result<(Vec<u8>, Vec<u8>), Denial> {
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
    Ok((checkpoint_bytes, source_root_bytes))
}

/// Before the first checkpoint `checkpoint.current` must still be absent: any
/// stream, even an empty one, is not the generation-zero basis.
fn observe_absent_checkpoint(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
) -> Result<(), Denial> {
    let observed = discovery
        .read(
            ArtifactCeiling::declared(StreamArtifact::CurrentCheckpoint, 0),
            ReadGrant::ceiling_only(),
        )
        .observed()
        .map_err(Denial::Discovery)?;
    if observed.into_bytes().is_some() {
        return Err(Denial::CheckpointBinding);
    }
    Ok(())
}
