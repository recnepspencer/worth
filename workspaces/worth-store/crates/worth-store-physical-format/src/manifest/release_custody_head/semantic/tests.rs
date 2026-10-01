use super::*;
use crate::{
    BlobGenerationPublicationV1, BlobReclaimDescriptorV2, OriginalDropReservationRequestV1,
    ReleasedDropCustodyV1, ReleasedDropPredecessorV1, ReleasedGenerationReclaimBasisV1,
};
use sha2::{Digest, Sha256};

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([9; 16], ordinal).unwrap()
}

fn sha(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn basis() -> ReleasedGenerationReclaimBasisV1 {
    let publication = BlobGenerationPublicationV1::new(
        [1; 16],
        [2; 16],
        [3; 16],
        1,
        record(10),
        [11; 32],
        131_072,
        [12; 32],
        65_536,
        [13; 32],
    )
    .unwrap();
    ReleasedGenerationReclaimBasisV1::new(
        publication,
        record(11),
        sha(&publication.encode()),
        [14; 32],
    )
    .unwrap()
}

struct Triple {
    entry: ReleaseCustodyHeadEntryV1,
    frames: ReleaseCustodyHeadControlIdentityV1,
    descriptor: BlobReclaimDescriptorV3,
    reservation: OriginalDropReservedV1,
    manifest: DropSetManifestV3,
}

fn triple(
    prior: Option<ReleaseCustodyHeadEntryV1>,
    dropped: Vec<PersistedRecordIdentity>,
) -> Triple {
    let source = basis();
    let (
        manifest_id,
        reservation_id,
        descriptor_id,
        source_generation,
        slot_generation,
        cumulative,
    ) = match prior {
        None => (record(20), record(21), record(22), 7, 5, 2),
        Some(_) => (record(30), record(31), record(32), 11, 9, 3),
    };
    let manifest = DropSetManifestV3::new(
        [1; 16],
        [4; 16],
        BlobReclaimSourceBasisV1::ReleasedGeneration(source),
        dropped,
        slot_generation,
    )
    .unwrap();
    let manifest_sha = sha(&manifest.encode());
    let request = OriginalDropReservationRequestV1::new([5; 32], [6; 32], 1, 40).unwrap();
    let reservation = OriginalDropReservedV1::new(
        [1; 16],
        [4; 16],
        manifest_id,
        manifest_sha,
        manifest.source_basis_digest(),
        slot_generation,
        source_generation,
        request,
    )
    .unwrap();
    let predecessor = prior.map(|head| {
        ReleasedDropPredecessorV1::new(head.descriptor_record(), head.descriptor_frame_sha256())
            .unwrap()
    });
    let base = BlobReclaimDescriptorV2::new(
        [1; 16],
        [4; 16],
        BlobReclaimSourceKind::ReleasedGeneration,
        manifest.source_basis_digest(),
        manifest_id,
        manifest_sha,
        manifest.count(),
        source_generation,
        source_generation + 1,
        predecessor,
        cumulative,
        false,
    )
    .unwrap();
    let custody = ReleasedDropCustodyV1::new(
        [7; 32], [8; 32], [9; 32], [10; 32], [11; 32], [12; 32], request,
    )
    .unwrap();
    let descriptor = BlobReclaimDescriptorV3::new(base, custody).unwrap();
    let descriptor_sha = sha(&descriptor.encode());
    let reservation_sha = sha(&reservation.encode());
    let entry = ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new(source.object(), source.generation()).unwrap(),
        descriptor_id,
        descriptor_sha,
        manifest_id,
        manifest_sha,
        reservation_id,
        reservation_sha,
        manifest.source_basis_digest(),
        predecessor,
        source_generation,
        cumulative,
        false,
    )
    .unwrap();
    let frames = ReleaseCustodyHeadControlIdentityV1::new(
        descriptor_id,
        descriptor_sha,
        reservation_id,
        reservation_sha,
        manifest_id,
        manifest_sha,
    )
    .unwrap();
    Triple {
        entry,
        frames,
        descriptor,
        reservation,
        manifest,
    }
}

fn verify_current(triple: &Triple) -> Result<(), ReleaseCustodyHeadDenial> {
    verify_release_custody_head_controls(
        triple.entry,
        triple.frames,
        triple.descriptor,
        triple.reservation,
        &triple.manifest,
    )
}

#[test]
fn current_and_successor_controls_match_exact_key_frames_and_progress() {
    let first = triple(None, vec![record(11), record(12)]);
    assert_eq!(verify_current(&first), Ok(()));
    let manifest_bytes = first.manifest.encode();
    let borrowed = DropSetManifestV3View::decode(&manifest_bytes).unwrap();
    assert_eq!(
        first.descriptor.canonical_frame_sha256(),
        sha(&first.descriptor.encode())
    );
    assert_eq!(
        first.reservation.canonical_frame_sha256(),
        sha(&first.reservation.encode())
    );
    assert_eq!(borrowed.canonical_frame_sha256(), sha(&manifest_bytes));
    assert_eq!(
        verify_release_custody_head_controls_view(
            first.entry,
            first.frames,
            first.descriptor,
            first.reservation,
            borrowed
        ),
        Ok(())
    );
    let next = triple(Some(first.entry), vec![record(13)]);
    assert_eq!(verify_current(&next), Ok(()));
    assert_eq!(
        verify_release_custody_head_successor(
            first.entry,
            next.entry,
            next.frames,
            next.descriptor,
            next.reservation,
            &next.manifest,
        ),
        Ok(())
    );
}

#[test]
fn first_release_missing_publication_and_cross_object_head_deny() {
    let first = triple(None, vec![record(12), record(13)]);
    assert_eq!(
        verify_current(&first),
        Err(ReleaseCustodyHeadDenial::Mutation)
    );
    let valid = triple(None, vec![record(11), record(12)]);
    let foreign = ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new([7; 16], 1).unwrap(),
        valid.entry.descriptor_record(),
        valid.entry.descriptor_frame_sha256(),
        valid.entry.manifest_record(),
        valid.entry.manifest_frame_sha256(),
        valid.entry.reservation_record(),
        valid.entry.reservation_frame_sha256(),
        valid.entry.source_basis_digest(),
        None,
        7,
        2,
        false,
    )
    .unwrap();
    assert_eq!(
        verify_release_custody_head_controls(
            foreign,
            valid.frames,
            valid.descriptor,
            valid.reservation,
            &valid.manifest,
        ),
        Err(ReleaseCustodyHeadDenial::Mutation)
    );
    // A first descriptor cannot claim more historical progress than its
    // authenticated first manifest, even before it reaches the head join.
    assert!(BlobReclaimDescriptorV2::new(
        [1; 16],
        [4; 16],
        BlobReclaimSourceKind::ReleasedGeneration,
        valid.entry.source_basis_digest(),
        valid.entry.manifest_record(),
        valid.entry.manifest_frame_sha256(),
        valid.manifest.count(),
        7,
        8,
        None,
        u64::from(valid.manifest.count()) + 1,
        false,
    )
    .is_err());
}

#[test]
fn wrong_frame_hash_and_predecessor_progress_deny() {
    let first = triple(None, vec![record(11), record(12)]);
    let next = triple(Some(first.entry), vec![record(13)]);
    let wrong_frames = ReleaseCustodyHeadControlIdentityV1::new(
        next.entry.descriptor_record(),
        [99; 32],
        next.entry.reservation_record(),
        next.entry.reservation_frame_sha256(),
        next.entry.manifest_record(),
        next.entry.manifest_frame_sha256(),
    )
    .unwrap();
    assert_eq!(
        verify_release_custody_head_controls(
            next.entry,
            wrong_frames,
            next.descriptor,
            next.reservation,
            &next.manifest,
        ),
        Err(ReleaseCustodyHeadDenial::Digest)
    );
    let terminal_prior = ReleaseCustodyHeadEntryV1::new(
        first.entry.key(),
        first.entry.descriptor_record(),
        first.entry.descriptor_frame_sha256(),
        first.entry.manifest_record(),
        first.entry.manifest_frame_sha256(),
        first.entry.reservation_record(),
        first.entry.reservation_frame_sha256(),
        first.entry.source_basis_digest(),
        first.entry.predecessor(),
        first.entry.source_root_generation(),
        first.entry.cumulative_dropped(),
        true,
    )
    .unwrap();
    assert_eq!(
        verify_release_custody_head_successor(
            terminal_prior,
            next.entry,
            next.frames,
            next.descriptor,
            next.reservation,
            &next.manifest,
        ),
        Err(ReleaseCustodyHeadDenial::Mutation)
    );
    let unrelated_prior = triple(None, vec![record(11), record(12)]).entry;
    let stale = ReleaseCustodyHeadEntryV1::new(
        unrelated_prior.key(),
        record(40),
        [40; 32],
        unrelated_prior.manifest_record(),
        unrelated_prior.manifest_frame_sha256(),
        unrelated_prior.reservation_record(),
        unrelated_prior.reservation_frame_sha256(),
        unrelated_prior.source_basis_digest(),
        None,
        7,
        2,
        false,
    )
    .unwrap();
    assert_eq!(
        verify_release_custody_head_successor(
            stale,
            next.entry,
            next.frames,
            next.descriptor,
            next.reservation,
            &next.manifest,
        ),
        Err(ReleaseCustodyHeadDenial::Mutation)
    );
}
