//! Pure agreement between a keyed head and decoded released-drop controls.
//! This does not witness selected routes, frame bytes, WAL fate, or custody.

use crate::{
    BlobReclaimDescriptorV3, BlobReclaimSourceBasisV1, BlobReclaimSourceKind, DropSetManifestV3,
    DropSetManifestV3View, OriginalDropReservedV1, PersistedRecordIdentity,
};

use super::{ReleaseCustodyHeadDenial, ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1};

/// Identities and hashes must come from independently witnessed control frames.
/// Constructing this value is not evidence that the frames exist on selected media.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseCustodyHeadControlIdentityV1 {
    descriptor_record: PersistedRecordIdentity,
    descriptor_frame_sha256: [u8; 32],
    reservation_record: PersistedRecordIdentity,
    reservation_frame_sha256: [u8; 32],
    manifest_record: PersistedRecordIdentity,
    manifest_frame_sha256: [u8; 32],
}

impl ReleaseCustodyHeadControlIdentityV1 {
    pub fn new(
        descriptor_record: PersistedRecordIdentity,
        descriptor_frame_sha256: [u8; 32],
        reservation_record: PersistedRecordIdentity,
        reservation_frame_sha256: [u8; 32],
        manifest_record: PersistedRecordIdentity,
        manifest_frame_sha256: [u8; 32],
    ) -> Result<Self, ReleaseCustodyHeadDenial> {
        if descriptor_record == reservation_record
            || descriptor_record == manifest_record
            || reservation_record == manifest_record
            || [
                descriptor_frame_sha256,
                reservation_frame_sha256,
                manifest_frame_sha256,
            ]
            .contains(&[0; 32])
        {
            return Err(ReleaseCustodyHeadDenial::Identity);
        }
        Ok(Self {
            descriptor_record,
            descriptor_frame_sha256,
            reservation_record,
            reservation_frame_sha256,
            manifest_record,
            manifest_frame_sha256,
        })
    }
}

/// Checks a current head against one exact decoded descriptor/reservation/manifest triple.
/// Frame digests are recomputed from canonical typed encoding, but the caller must
/// independently establish the identities, hashes, and bytes on selected media.
pub fn verify_release_custody_head_controls(
    entry: ReleaseCustodyHeadEntryV1,
    frames: ReleaseCustodyHeadControlIdentityV1,
    descriptor: BlobReclaimDescriptorV3,
    reservation: OriginalDropReservedV1,
    manifest: &DropSetManifestV3,
) -> Result<(), ReleaseCustodyHeadDenial> {
    verify_release_custody_head_controls_view(
        entry,
        frames,
        descriptor,
        reservation,
        DropSetManifestV3View::from_owned(manifest),
    )
}

/// The same semantic predicate over a grammar-validated borrowed control
/// frame. Neither variant supplies selected-media authority.
pub fn verify_release_custody_head_controls_view(
    entry: ReleaseCustodyHeadEntryV1,
    frames: ReleaseCustodyHeadControlIdentityV1,
    descriptor: BlobReclaimDescriptorV3,
    reservation: OriginalDropReservedV1,
    manifest: DropSetManifestV3View<'_>,
) -> Result<(), ReleaseCustodyHeadDenial> {
    let base = descriptor.base();
    let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis() else {
        return Err(ReleaseCustodyHeadDenial::Mutation);
    };
    if frames.descriptor_record != entry.descriptor_record()
        || frames.reservation_record != entry.reservation_record()
        || frames.manifest_record != entry.manifest_record()
        || base.manifest_record() != frames.manifest_record
        || reservation.manifest_record() != frames.manifest_record
    {
        return Err(ReleaseCustodyHeadDenial::Identity);
    }
    if frames.descriptor_frame_sha256 != entry.descriptor_frame_sha256()
        || frames.reservation_frame_sha256 != entry.reservation_frame_sha256()
        || frames.manifest_frame_sha256 != entry.manifest_frame_sha256()
        || base.manifest_frame_sha256() != frames.manifest_frame_sha256
        || reservation.manifest_frame_sha256() != frames.manifest_frame_sha256
        || descriptor.canonical_frame_sha256() != frames.descriptor_frame_sha256
        || reservation.canonical_frame_sha256() != frames.reservation_frame_sha256
        || manifest.canonical_frame_sha256() != frames.manifest_frame_sha256
    {
        return Err(ReleaseCustodyHeadDenial::Digest);
    }
    let store = source.publication().store();
    if ReleaseCustodyHeadKeyV1::new(source.object(), source.generation()) != Some(entry.key())
        || base.store() != store
        || reservation.store() != store
        || manifest.store() != store
        || base.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
        || base.source_basis_digest() != entry.source_basis_digest()
        || reservation.source_basis_digest() != entry.source_basis_digest()
        || manifest.source_basis_digest() != entry.source_basis_digest()
        || base.reclaim_attempt() != reservation.reclaim_attempt()
        || base.reclaim_attempt() != manifest.reclaim_attempt()
        || base.manifest_count() != manifest.count()
        || base.predecessor() != entry.predecessor()
        || base.source_root_generation() != entry.source_root_generation()
        || base.cumulative_dropped() != entry.cumulative_dropped()
        || base.terminal() != entry.terminal()
        || reservation.reserved_selected_generation() != base.source_root_generation()
        || reservation.manifest_selected_generation() != manifest.never_reserved_slot_generation()
        || reservation.request() != descriptor.custody().request()
    {
        return Err(ReleaseCustodyHeadDenial::Mutation);
    }
    let publication_dropped = manifest.contains_record(source.publication_record());
    if publication_dropped != base.predecessor().is_none()
        || (base.predecessor().is_none()
            && base.cumulative_dropped() != u64::from(manifest.count()))
    {
        return Err(ReleaseCustodyHeadDenial::Mutation);
    }
    Ok(())
}

/// Additional one-key progression check for a new WAL-backed head replacement.
/// A current checkpoint head may have a pruned predecessor; only call this when
/// an authenticated prior head is actually available from the source tree.
pub fn verify_release_custody_head_successor(
    prior: ReleaseCustodyHeadEntryV1,
    next: ReleaseCustodyHeadEntryV1,
    frames: ReleaseCustodyHeadControlIdentityV1,
    descriptor: BlobReclaimDescriptorV3,
    reservation: OriginalDropReservedV1,
    manifest: &DropSetManifestV3,
) -> Result<(), ReleaseCustodyHeadDenial> {
    verify_release_custody_head_controls(next, frames, descriptor, reservation, manifest)?;
    let predecessor = next
        .predecessor()
        .ok_or(ReleaseCustodyHeadDenial::Mutation)?;
    if prior.terminal()
        || prior.key() != next.key()
        || prior.source_basis_digest() != next.source_basis_digest()
        || next.descriptor_record() == prior.descriptor_record()
        || predecessor.descriptor_record() != prior.descriptor_record()
        || predecessor.descriptor_frame_sha256() != prior.descriptor_frame_sha256()
        || next.source_root_generation() <= prior.source_root_generation()
        || prior
            .cumulative_dropped()
            .checked_add(u64::from(manifest.count()))
            != Some(next.cumulative_dropped())
    {
        return Err(ReleaseCustodyHeadDenial::Mutation);
    }
    Ok(())
}

#[cfg(test)]
#[path = "semantic/tests.rs"]
mod tests;
