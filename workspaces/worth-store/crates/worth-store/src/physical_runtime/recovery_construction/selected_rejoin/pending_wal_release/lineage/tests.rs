use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobReclaimDescriptorV2, BlobReclaimDescriptorV3,
    BlobReclaimSourceBasisV1, DropSetManifestV3, OriginalDropReservationRequestV1,
    PersistedRecordIdentity, PhysicalCheckpointIdentity, ReleaseCheckpointBatchV1,
    ReleaseCheckpointCertificateV1, ReleasedDropCustodyV1, ReleasedDropPredecessorV1,
    ReleasedDropWalFateWitnessV1, ReleasedGenerationReclaimBasisV1,
};

use super::{same_source_and_cumulative, selected_predecessor_matches};
use crate::physical_runtime::recovery_construction::selected_rejoin::control_frames::catalog::decode_predecessor_frames;

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).unwrap()
}

fn released_source(object: u8) -> BlobReclaimSourceBasisV1 {
    let publication = BlobGenerationPublicationV1::new(
        [7; 16],
        [object; 16],
        [3; 16],
        3,
        record(3),
        [4; 32],
        8,
        [5; 32],
        64 << 10,
        [6; 32],
    )
    .unwrap();
    BlobReclaimSourceBasisV1::ReleasedGeneration(
        ReleasedGenerationReclaimBasisV1::new(
            publication,
            record(4),
            Sha256::digest(publication.encode()).into(),
            [6; 32],
        )
        .unwrap(),
    )
}

fn descriptor(
    manifest: &DropSetManifestV3,
    manifest_record: PersistedRecordIdentity,
    source_root: u64,
    predecessor: Option<ReleasedDropPredecessorV1>,
    cumulative: u64,
) -> BlobReclaimDescriptorV3 {
    descriptor_with_terminal(
        manifest,
        manifest_record,
        source_root,
        predecessor,
        cumulative,
        false,
    )
}

fn descriptor_with_terminal(
    manifest: &DropSetManifestV3,
    manifest_record: PersistedRecordIdentity,
    source_root: u64,
    predecessor: Option<ReleasedDropPredecessorV1>,
    cumulative: u64,
    terminal: bool,
) -> BlobReclaimDescriptorV3 {
    let base = BlobReclaimDescriptorV2::new(
        manifest.store(),
        manifest.reclaim_attempt(),
        manifest.source_kind(),
        manifest.source_basis_digest(),
        manifest_record,
        Sha256::digest(manifest.encode()).into(),
        manifest.count(),
        source_root,
        source_root + 1,
        predecessor,
        cumulative,
        terminal,
    )
    .unwrap();
    let request = OriginalDropReservationRequestV1::new([13; 32], [14; 32], 9, 20).unwrap();
    let custody = ReleasedDropCustodyV1::new(
        [7; 32], [8; 32], [9; 32], [10; 32], [11; 32], [12; 32], request,
    )
    .unwrap();
    BlobReclaimDescriptorV3::new(base, custody).unwrap()
}

fn checkpoint() -> PhysicalCheckpointIdentity {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap(),
    )
    .published_identity();
    PhysicalCheckpointIdentity::new(store, NonZeroU64::new(3).unwrap())
}

#[test]
fn selected_batch_catalog_rejects_cross_object_predecessor_and_accepts_same_object() {
    let prior_manifest =
        DropSetManifestV3::new([7; 16], [8; 16], released_source(0x21), vec![record(3)], 9)
            .unwrap();
    let prior = descriptor(&prior_manifest, record(10), 10, None, 1);
    let prior_bytes = prior.encode();
    let prior_manifest_bytes = prior_manifest.encode();
    let prior_sha: [u8; 32] = Sha256::digest(&prior_bytes).into();
    let predecessor = ReleasedDropPredecessorV1::new(record(11), prior_sha).unwrap();
    let batch = ReleaseCheckpointBatchV1::new(
        checkpoint(),
        12,
        [1; 32],
        0,
        record(11),
        prior_sha,
        prior.custody_digest(),
        record(12),
        [2; 32],
        prior.custody().request(),
        ReleasedDropWalFateWitnessV1::new(100, 110, [7; 32], [8; 32]).unwrap(),
        11,
        [9; 32],
        None,
        1,
        [3; 32],
        false,
    )
    .unwrap();
    assert_eq!(
        ReleaseCheckpointCertificateV1::decode(
            &ReleaseCheckpointCertificateV1::Batch(batch).encode()
        ),
        Ok(ReleaseCheckpointCertificateV1::Batch(batch))
    );
    let frames = [
        (record(11), prior_sha, prior_bytes.as_slice()),
        (
            record(10),
            Sha256::digest(&prior_manifest_bytes).into(),
            prior_manifest_bytes.as_slice(),
        ),
    ];
    let selected_lookup =
        |record, sha| decode_predecessor_frames(frames.iter().copied(), record, sha);
    let same_manifest =
        DropSetManifestV3::new([7; 16], [8; 16], released_source(0x21), vec![record(5)], 9)
            .unwrap();
    let same = descriptor(&same_manifest, record(13), 12, Some(predecessor), 2);
    assert!(selected_predecessor_matches(
        &[batch],
        None,
        same,
        &same_manifest,
        selected_lookup
    ));
    let foreign_manifest =
        DropSetManifestV3::new([7; 16], [8; 16], released_source(0x22), vec![record(5)], 9)
            .unwrap();
    let foreign = descriptor(&foreign_manifest, record(14), 12, Some(predecessor), 2);
    assert!(!selected_predecessor_matches(
        &[batch],
        None,
        foreign,
        &foreign_manifest,
        selected_lookup
    ));
    assert!(selected_predecessor_matches(
        &[],
        Some((record(11), prior_sha)),
        same,
        &same_manifest,
        selected_lookup
    ));
    assert!(!selected_predecessor_matches(
        &[],
        Some((record(11), [0; 32])),
        same,
        &same_manifest,
        selected_lookup
    ));
    assert!(!selected_predecessor_matches(
        &[],
        Some((record(11), prior_sha)),
        foreign,
        &foreign_manifest,
        selected_lookup
    ));
    let terminal_prior = descriptor_with_terminal(&prior_manifest, record(10), 10, None, 1, true);
    let terminal_bytes = terminal_prior.encode();
    let terminal_sha: [u8; 32] = Sha256::digest(&terminal_bytes).into();
    let terminal_predecessor = ReleasedDropPredecessorV1::new(record(11), terminal_sha).unwrap();
    let terminal_successor = descriptor(
        &same_manifest,
        record(13),
        12,
        Some(terminal_predecessor),
        2,
    );
    let terminal_frames = [
        (record(11), terminal_sha, terminal_bytes.as_slice()),
        (
            record(10),
            Sha256::digest(&prior_manifest_bytes).into(),
            prior_manifest_bytes.as_slice(),
        ),
    ];
    assert!(!selected_predecessor_matches(
        &[],
        Some((record(11), terminal_sha)),
        terminal_successor,
        &same_manifest,
        |record, sha| decode_predecessor_frames(terminal_frames.iter().copied(), record, sha),
    ));
}

#[test]
fn different_released_object_cannot_borrow_certified_batch_predecessor() {
    let prior = released_source(0x21);
    let same = released_source(0x21);
    let foreign = released_source(0x22);
    assert!(same_source_and_cumulative(prior, same, 1, 1, 2));
    assert!(!same_source_and_cumulative(prior, foreign, 1, 1, 2));
    assert!(!same_source_and_cumulative(prior, same, 1, 1, 3));
}
