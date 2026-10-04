use sha2::{Digest, Sha256};

use super::*;
use crate::{
    decode_blob_record, BlobGenerationPublicationV1, BlobRecordDenial, BlobRecordV1,
    PersistedRecordIdentity, BLOB_RECORD_HEADER_BYTES,
};

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).expect("valid record")
}

fn release_basis() -> ReleasedGenerationReclaimBasisV1 {
    let publication = BlobGenerationPublicationV1::new(
        [7; 16],
        [2; 16],
        [3; 16],
        3,
        record(3),
        [4; 32],
        8,
        [5; 32],
        64 * 1024,
        [6; 32],
    )
    .unwrap();
    ReleasedGenerationReclaimBasisV1::new(
        publication,
        record(4),
        Sha256::digest(publication.encode()).into(),
        [6; 32],
    )
    .unwrap()
}

#[test]
fn released_source_basis_has_exact_canonical_wal_witness_bytes() {
    let basis = release_basis();
    let encoded = basis.encode();
    assert_eq!(
        ReleasedGenerationReclaimBasisV1::decode(&encoded),
        Ok(basis)
    );
    assert_eq!(basis.encode(), encoded);
    assert!(ReleasedGenerationReclaimBasisV1::decode(&encoded[..encoded.len() - 1]).is_err());
    let mut changed = encoded;
    changed[24] ^= 1;
    assert!(ReleasedGenerationReclaimBasisV1::decode(&changed).is_err());
}

#[test]
fn released_generation_manifest_and_descriptor_round_trip_with_exact_source_kind() {
    let source = BlobReclaimSourceBasisV1::ReleasedGeneration(release_basis());
    let manifest =
        DropSetManifestV3::new([7; 16], [8; 16], source, vec![record(4), record(5)], 9).unwrap();
    let bytes = manifest.encode();
    assert_eq!(bytes.len(), manifest.encoded_frame_bytes());
    assert_eq!(bytes[8], 13);
    assert_eq!(bytes[BLOB_RECORD_HEADER_BYTES + 32], 2);
    assert_eq!(DropSetManifestV3::decode(&bytes), Ok(manifest.clone()));
    let borrowed = DropSetManifestV3View::decode(&bytes).unwrap();
    assert_eq!(
        borrowed.canonical_frame_sha256(),
        <[u8; 32]>::from(Sha256::digest(&bytes))
    );
    assert_eq!(
        borrowed.source_basis_digest(),
        manifest.source_basis_digest()
    );
    assert!(borrowed.contains_record(record(4)));
    assert!(!borrowed.contains_record(record(6)));
    assert!(
        matches!(decode_blob_record(&bytes), Ok(BlobRecordV1::DropSetManifestV3(value)) if value == manifest)
    );
    assert_eq!(manifest.count(), 2);
    let descriptor = BlobReclaimDescriptorV2::new(
        manifest.store(),
        manifest.reclaim_attempt(),
        manifest.source_kind(),
        manifest.source_basis_digest(),
        record(6),
        Sha256::digest(&bytes).into(),
        manifest.count(),
        10,
        11,
        None,
        u64::from(manifest.count()),
        true,
    )
    .unwrap();
    let descriptor_bytes = descriptor.encode();
    assert_eq!(descriptor_bytes[8], 14);
    assert_eq!(descriptor_bytes[BLOB_RECORD_HEADER_BYTES + 32], 2);
    assert_eq!(
        BlobReclaimDescriptorV2::decode(&descriptor_bytes),
        Ok(descriptor)
    );
    assert!(
        matches!(decode_blob_record(&descriptor_bytes), Ok(BlobRecordV1::ReclaimDescriptorV2(value)) if value == descriptor)
    );
    assert_ne!(source.digest([7; 16]), source.digest([9; 16]));
}

#[test]
fn borrowed_v3_manifest_rejects_reordered_and_forged_dropped_identities() {
    let source = BlobReclaimSourceBasisV1::ReleasedGeneration(release_basis());
    let manifest =
        DropSetManifestV3::new([7; 16], [8; 16], source, vec![record(4), record(5)], 9).unwrap();
    let encoded = manifest.encode();
    let mut payload = encoded[BLOB_RECORD_HEADER_BYTES..].to_vec();
    let dropped_start = 35 + source.encoded_len() + 34;
    payload[dropped_start + 16..dropped_start + 24].copy_from_slice(&5_u64.to_le_bytes());
    // Both digests agree with the duplicate identities, so only canonical
    // ordering may deny this payload.
    payload[dropped_start - 32..dropped_start].copy_from_slice(
        &super::drop_set_manifest::digest_dropped(&[record(5), record(5)]),
    );
    let first =
        super::super::envelope::encode(crate::BlobRecordKind::DropSetManifestV3, &payload).unwrap();
    assert!(matches!(
        DropSetManifestV3View::decode(&first),
        Err(BlobRecordDenial::InvalidDropSet)
    ));
    let mut payload = encoded[BLOB_RECORD_HEADER_BYTES..].to_vec();
    payload[dropped_start] ^= 1;
    let forged =
        super::super::envelope::encode(crate::BlobRecordKind::DropSetManifestV3, &payload).unwrap();
    assert!(matches!(
        DropSetManifestV3View::decode(&forged),
        Err(BlobRecordDenial::InvalidDropSet)
    ));
}

#[test]
fn release_manifest_allows_continuation_but_rejects_foreign_store_and_malformed_tag() {
    let source = BlobReclaimSourceBasisV1::ReleasedGeneration(release_basis());
    assert!(DropSetManifestV3::new([7; 16], [8; 16], source, vec![record(5)], 9).is_ok());
    assert_eq!(
        DropSetManifestV3::new([9; 16], [8; 16], source, vec![record(4)], 9),
        Err(BlobRecordDenial::InvalidDropSet)
    );
    let manifest = DropSetManifestV3::new([7; 16], [8; 16], source, vec![record(4)], 9).unwrap();
    let bytes = manifest.encode();
    let mut payload = bytes[BLOB_RECORD_HEADER_BYTES..].to_vec();
    payload[32] = 3;
    assert_eq!(
        DropSetManifestV3::decode_payload(&payload),
        Err(BlobRecordDenial::InvalidReclaimSource)
    );
    let last = payload.len() - 1;
    payload[32] = 2;
    payload[last] = 2;
    assert!(DropSetManifestV3::decode_payload(&payload).is_err());
    payload[last] = 1;
    payload[33] = 127;
    assert!(DropSetManifestV3::decode_payload(&payload).is_err());
}

#[test]
fn a_v3_manifest_prefix_names_its_source_without_its_dropped_identities() {
    let source = BlobReclaimSourceBasisV1::ReleasedGeneration(release_basis());
    let dropped = (4..64).map(record).collect();
    let manifest = DropSetManifestV3::new([7; 16], [8; 16], source, dropped, 9).unwrap();
    let frame = manifest.encode();
    let window = DropSetManifestV3View::SOURCE_BASIS_PREFIX_BYTES;
    assert_eq!(window, BLOB_RECORD_HEADER_BYTES + 35 + source.encoded_len());
    assert!(window < frame.len());
    assert_eq!(
        DropSetManifestV3View::source_basis_in_prefix(&frame[..window]),
        Ok(source)
    );
    assert_eq!(
        DropSetManifestV3View::source_basis_in_prefix(&frame),
        Ok(source)
    );
    assert_eq!(
        DropSetManifestV3View::source_basis_in_prefix(&frame[..window - 1]),
        Err(BlobRecordDenial::LengthMismatch)
    );
    assert_eq!(
        DropSetManifestV3View::source_basis_in_prefix(&frame[..BLOB_RECORD_HEADER_BYTES - 1]),
        Err(BlobRecordDenial::Truncated)
    );
    // The basis authenticates its own publication frame: one changed session
    // byte is a denial, never another session's name.
    let mut forged = frame[..window].to_vec();
    forged[window - 100] ^= 1;
    assert!(DropSetManifestV3View::source_basis_in_prefix(&forged).is_err());
    for (offset, denial) in [
        (0, BlobRecordDenial::WrongMagic),
        (8, BlobRecordDenial::UnknownKind),
        (9, BlobRecordDenial::UnsupportedVersion),
    ] {
        let mut other = frame[..window].to_vec();
        other[offset] ^= 0x40;
        assert_eq!(
            DropSetManifestV3View::source_basis_in_prefix(&other),
            Err(denial)
        );
    }
}

#[test]
fn continuation_descriptor_binds_predecessor_and_cumulative_count() {
    let predecessor = ReleasedDropPredecessorV1::new(record(8), [9; 32]).unwrap();
    let descriptor = BlobReclaimDescriptorV2::new(
        [7; 16],
        [8; 16],
        BlobReclaimSourceKind::ReleasedGeneration,
        [1; 32],
        record(10),
        [2; 32],
        2,
        20,
        21,
        Some(predecessor),
        5,
        false,
    )
    .unwrap();
    assert_eq!(descriptor.predecessor(), Some(predecessor));
    assert_eq!(descriptor.cumulative_dropped(), 5);
    assert!(!descriptor.terminal());
    assert_eq!(
        BlobReclaimDescriptorV2::decode(&descriptor.encode()),
        Ok(descriptor)
    );
    assert!(BlobReclaimDescriptorV2::new(
        [7; 16],
        [8; 16],
        BlobReclaimSourceKind::ReleasedGeneration,
        [1; 32],
        record(10),
        [2; 32],
        2,
        20,
        21,
        None,
        5,
        false,
    )
    .is_err());
    let mut tampered = descriptor.encode()[BLOB_RECORD_HEADER_BYTES..].to_vec();
    tampered[139] = 0;
    assert!(BlobReclaimDescriptorV2::decode_payload(&tampered).is_err());
}

#[test]
fn legacy_failed_ingest_manifest_and_descriptor_bytes_remain_distinct() {
    let basis =
        FailedIngestReclaimBasisV1::new([2; 16], record(1), [3; 32], record(2), [4; 32]).unwrap();
    let legacy = DropSetManifestV2::new([5; 16], [6; 16], basis, vec![record(3)], 7).unwrap();
    let legacy_bytes = legacy.encode();
    assert_eq!(legacy_bytes[8], 9);
    assert_eq!(DropSetManifestV2::decode(&legacy_bytes), Ok(legacy));
    assert!(DropSetManifestV3::decode(&legacy_bytes).is_err());
    let descriptor = BlobReclaimDescriptorV1::new(
        [5; 16],
        [6; 16],
        basis.digest([5; 16]),
        record(4),
        [7; 32],
        1,
        8,
        9,
    )
    .unwrap();
    let descriptor_bytes = descriptor.encode();
    assert_eq!(descriptor_bytes[8], 8);
    assert_eq!(
        BlobReclaimDescriptorV1::decode(&descriptor_bytes),
        Ok(descriptor)
    );
    assert!(BlobReclaimDescriptorV2::decode(&descriptor_bytes).is_err());
}
