//! Fresh current selector/checkpoint and ordinary selected root observations.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::{
    ArtifactCeiling, BoundedRecoveryFilesystemDiscovery, FixedArtifact, PageAddress,
};
use worth_store_physical_format::{
    checkpoint_stream_encoded_digest, decode_checkpoint_certificate, maximum_current_root_entries,
    CheckpointCertificateKind, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    DurableRootSelector, PhysicalCheckpointSource, ReleaseCheckpointCertificateV1,
    RootSelectorRole, CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
    RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES,
};
use worth_store_recovery_physics::VerifiedOrderedHistoricalReleaseCustody;

use super::super::resident::StoreRejoinResidentLedger;
use super::super::{
    claimed_checkpoint, tier, SelectedMediaRejoinDenial as Denial, MAX_CHECKPOINT_BYTES,
};
use crate::physical_runtime::{
    CompletedPhysicalRecoveryFreshReopen, FundedRecoveryObservation, PhysicalRecoveryReadAllocation,
};

pub(super) struct Selection {
    selector: FundedRecoveryObservation,
    root: FundedRecoveryObservation,
    free: FundedRecoveryObservation,
    checkpoint: FundedRecoveryObservation,
    checkpoint_source: FundedRecoveryObservation,
    pub(super) manifest: DurablePhysicalRootManifest,
    pub(super) free_header: DurableFreeSpaceManifestHeader,
    pub(super) checkpoint_source_root: DurablePhysicalRootManifest,
}

impl Selection {
    pub(super) fn same_bytes(&self, other: &Self) -> bool {
        self.selector.observed().bytes() == other.selector.observed().bytes()
            && self.root.observed().bytes() == other.root.observed().bytes()
            && self.free.observed().bytes() == other.free.observed().bytes()
            && self.checkpoint.observed().bytes() == other.checkpoint.observed().bytes()
            && self.checkpoint_source.observed().bytes()
                == other.checkpoint_source.observed().bytes()
    }

    pub(super) fn owned_heap_bytes(&self) -> Option<u64> {
        [
            &self.selector,
            &self.root,
            &self.free,
            &self.checkpoint,
            &self.checkpoint_source,
        ]
        .into_iter()
        .try_fold(0_u64, |sum, observation| {
            sum.checked_add(observation.owned_heap_bytes()?)
        })
    }

    pub(super) fn discard(self, resident: &mut StoreRejoinResidentLedger) -> Result<(), Denial> {
        let bytes = self.owned_heap_bytes().ok_or(Denial::BoundExceeded)?;
        drop(self);
        resident.release(bytes);
        Ok(())
    }
}

pub(super) fn observe(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedOrderedHistoricalReleaseCustody,
    checkpoint_stream: &worth_store_physical_integrity::VerifiedCheckpointStream,
    window: &mut PhysicalRecoveryReadAllocation<'_>,
    resident: &mut StoreRejoinResidentLedger,
) -> Result<Selection, Denial> {
    if claim.checkpoint().encoded_bytes() > MAX_CHECKPOINT_BYTES {
        return Err(Denial::BoundExceeded);
    }
    let format = reopen.format();
    let selector = funded_record(
        discovery,
        window,
        resident,
        ArtifactCeiling::fixed(FixedArtifact::CurrentRootSelector),
    )?;
    let selector_bytes = selector.observed().bytes().ok_or(Denial::MissingSelector)?;
    let decoded = DurableRootSelector::decode(selector_bytes).map_err(|_| Denial::RootBinding)?;
    if decoded.encode() != selector_bytes
        || decoded.store_identity() != discovery.store_identity()
        || decoded.format() != format
        || decoded.role() != RootSelectorRole::Current
        || decoded.root_generation() != claim.selected_root().generation()
        || selector_bytes != reopen.fresh_reopen_occurrence().selector().bytes()
    {
        return Err(Denial::RootBinding);
    }
    let root = funded_record(
        discovery,
        window,
        resident,
        ArtifactCeiling::page(
            format,
            PageAddress::RootManifest {
                generation: decoded.root_generation(),
            },
        ),
    )?;
    let root_bytes = root.observed().bytes().ok_or(Denial::MissingRoot)?;
    let (manifest, root_format) =
        DurablePhysicalRootManifest::decode(root_bytes, claim.selected_root().node_capacity())
            .map_err(|_| Denial::RootBinding)?;
    if root_format != format
        || manifest != *claim.selected_root()
        || manifest != *reopen.root()
        || !canonical_eq(
            window,
            resident,
            manifest.encoded_frame_bytes(),
            root_bytes,
            |buffer| manifest.encode_in_reserved(format, buffer),
        )?
        || root_bytes != reopen.fresh_reopen_occurrence().root().bytes()
        || <[u8; 32]>::from(Sha256::digest(root_bytes)) != claim.selected_root_frame_sha256()
    {
        return Err(Denial::RootBinding);
    }
    let free = funded_record(
        discovery,
        window,
        resident,
        ArtifactCeiling::page(
            format,
            PageAddress::FreeSpaceManifest {
                generation: manifest.generation(),
            },
        ),
    )?;
    let free_bytes = free.observed().bytes().ok_or(Denial::MissingFrame)?;
    let free_header = tier::selection::validate_selected_free_header(
        free_bytes,
        discovery.store_identity(),
        format,
        &manifest,
    )?;
    if free_header.tier_epoch_start().is_some() != manifest.tier_epoch_anchor().is_some()
        || !canonical_eq(
            window,
            resident,
            free_header.encoded_frame_bytes(),
            free_bytes,
            |buffer| free_header.encode_in_reserved(format, buffer),
        )?
        || <[u8; 32]>::from(Sha256::digest(free_bytes)) != claim.selected_free_space_frame_sha256()
    {
        return Err(Denial::RootBinding);
    }
    let checkpoint = window
        .read_record(
            discovery,
            claimed_checkpoint(claim.checkpoint().encoded_bytes())?,
        )
        .map_err(read_denial)?;
    resident
        .retain(checkpoint.owned_heap_bytes().ok_or(Denial::BoundExceeded)?)
        .map_err(Denial::Resident)?;
    let checkpoint_bytes = checkpoint
        .observed()
        .bytes()
        .ok_or(Denial::MissingCheckpoint)?;
    let source = PhysicalCheckpointSource::decode_stream_header_record(
        checkpoint_bytes
            .get(..CHECKPOINT_STREAM_HEADER_RECORD_BYTES)
            .ok_or(Denial::CheckpointBinding)?,
    )
    .map_err(|_| Denial::CheckpointBinding)?;
    if checkpoint_bytes.len() as u64 != claim.checkpoint().encoded_bytes()
        || checkpoint_stream_encoded_digest(checkpoint_bytes) != claim.checkpoint().encoded_digest()
        || source != claim.checkpoint().source()
        || source.root().generation() > manifest.generation()
    {
        return Err(Denial::CheckpointBinding);
    }
    let base_matches = match (claim.marker(), claim.selected_head_v2()) {
        (Some(marker), None) => {
            source.identity() == marker.checkpoint()
                && source.root().generation() == marker.root_generation()
                && marker.root_sha256() == claim.history().checkpoint_root_frame_sha256()
                && no_release_roster_matches(marker, checkpoint_stream, window, resident)?
        }
        // A released checkpoint base keeps its own checkpoint-source roster;
        // the completed history continues from it without replacing it.
        (None, Some(base)) => {
            *base.checkpoint() == *claim.checkpoint()
                && base.source_root_sha256() == claim.history().checkpoint_root_frame_sha256()
                && base.certificates_match(
                    checkpoint_stream
                        .certificate_records()
                        .iter()
                        .map(AsRef::as_ref),
                )
        }
        _ => false,
    };
    if !base_matches {
        return Err(Denial::CheckpointBinding);
    }
    let checkpoint_source = funded_record(
        discovery,
        window,
        resident,
        ArtifactCeiling::page(
            format,
            PageAddress::RootManifest {
                generation: source.root().generation(),
            },
        ),
    )?;
    let checkpoint_source_bytes = checkpoint_source
        .observed()
        .bytes()
        .ok_or(Denial::MissingRoot)?;
    let (checkpoint_source_root, checkpoint_format) = DurablePhysicalRootManifest::decode(
        checkpoint_source_bytes,
        maximum_current_root_entries(format),
    )
    .map_err(|_| Denial::CheckpointBinding)?;
    if checkpoint_format != format
        || checkpoint_source_root.generation() != source.root().generation()
        || checkpoint_source_root.tree_identity() != source.root().tree_identity()
        || !canonical_eq(
            window,
            resident,
            checkpoint_source_root.encoded_frame_bytes(),
            checkpoint_source_bytes,
            |buffer| checkpoint_source_root.encode_in_reserved(format, buffer),
        )?
        || <[u8; 32]>::from(Sha256::digest(checkpoint_source_bytes))
            != claim.history().checkpoint_root_frame_sha256()
        || claim
            .selected_head_v2()
            .is_some_and(|base| base.checkpoint_source_root() != &checkpoint_source_root)
    {
        return Err(Denial::CheckpointBinding);
    }
    Ok(Selection {
        selector,
        root,
        free,
        checkpoint,
        checkpoint_source,
        manifest,
        free_header,
        checkpoint_source_root,
    })
}

fn funded_record(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    window: &mut PhysicalRecoveryReadAllocation<'_>,
    resident: &mut StoreRejoinResidentLedger,
    ceiling: ArtifactCeiling,
) -> Result<FundedRecoveryObservation, Denial> {
    let observation = window
        .read_record(discovery, ceiling)
        .map_err(read_denial)?;
    resident
        .retain(
            observation
                .owned_heap_bytes()
                .ok_or(Denial::BoundExceeded)?,
        )
        .map_err(Denial::Resident)?;
    Ok(observation)
}

fn read_denial(
    failure: worth_store_physical_backend::RecoveryDiscoveryAllocationFailure<
        crate::physical_runtime::PhysicalRecoveryObservationAllocationDenial,
    >,
) -> Denial {
    super::native_storage::read_denial(failure)
}

fn no_release_roster_matches(
    expected: worth_store_physical_format::ReleaseCheckpointNoReleaseV1,
    checkpoint: &worth_store_physical_integrity::VerifiedCheckpointStream,
    window: &PhysicalRecoveryReadAllocation<'_>,
    resident: &mut StoreRejoinResidentLedger,
) -> Result<bool, Denial> {
    let [frame] = checkpoint.certificate_records() else {
        return Ok(false);
    };
    let Ok((CheckpointCertificateKind::ReleasedDrop, payload)) =
        decode_checkpoint_certificate(frame)
    else {
        return Ok(false);
    };
    let Ok(ReleaseCheckpointCertificateV1::NoRelease(marker)) =
        ReleaseCheckpointCertificateV1::decode(payload)
    else {
        return Ok(false);
    };
    Ok(marker == expected
        && canonical_eq(
            window,
            resident,
            RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES,
            payload,
            |buffer| marker.encode_in_reserved(buffer),
        )?)
}

fn canonical_eq(
    window: &PhysicalRecoveryReadAllocation<'_>,
    resident: &mut StoreRejoinResidentLedger,
    length: usize,
    observed: &[u8],
    encode: impl FnOnce(Vec<u8>) -> Option<Vec<u8>>,
) -> Result<bool, Denial> {
    use crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial as ResidentDenial;
    let requested = u64::try_from(length).map_err(|_| Denial::BoundExceeded)?;
    resident.transient(requested).map_err(Denial::Resident)?;
    let grant = window.reserve_owned(requested).map_err(Denial::Resident)?;
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(length)
        .map_err(|cause| Denial::Resident(ResidentDenial::Allocation { requested, cause }))?;
    let actual = buffer.capacity() as u64;
    if actual != requested {
        return Err(Denial::Resident(
            ResidentDenial::AllocatorExceededReservation { requested, actual },
        ));
    }
    let encoded = encode(buffer).ok_or(Denial::BoundExceeded)?;
    let matches = encoded.as_slice() == observed;
    drop(encoded);
    drop(grant);
    Ok(matches)
}
