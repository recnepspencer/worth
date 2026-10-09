use super::super::super::envelope::encode;
use super::*;
use crate::{decode_blob_record, BlobRecordV1, PersistedRecordIdentity};

fn descriptor(kind: BlobReclaimSourceKind) -> BlobReclaimDescriptorV2 {
    BlobReclaimDescriptorV2::new(
        [1; 16],
        [2; 16],
        kind,
        [3; 32],
        PersistedRecordIdentity::new([4; 16], 5).unwrap(),
        [6; 32],
        2,
        7,
        8,
        None,
        2,
        true,
    )
    .unwrap()
}

fn custody() -> ReleasedDropCustodyV1 {
    let request = OriginalDropReservationRequestV1::new([13; 32], [14; 32], 9, 12).unwrap();
    ReleasedDropCustodyV1::new(
        [7; 32], [8; 32], [9; 32], [10; 32], [11; 32], [12; 32], request,
    )
    .unwrap()
}

#[test]
fn released_drop_custody_roundtrips_exact_v2_prefix_and_domain_digest() {
    let value = BlobReclaimDescriptorV3::new(
        descriptor(BlobReclaimSourceKind::ReleasedGeneration),
        custody(),
    )
    .unwrap();
    let bytes = value.encode();
    assert_eq!(bytes.len(), BLOB_RECORD_HEADER_BYTES + PAYLOAD_BYTES);
    assert_eq!(
        &bytes[BLOB_RECORD_HEADER_BYTES..BLOB_RECORD_HEADER_BYTES + V2_PAYLOAD_BYTES],
        &value.base().encode()[BLOB_RECORD_HEADER_BYTES..]
    );
    assert_eq!(BlobReclaimDescriptorV3::decode(&bytes), Ok(value));
    assert_eq!(
        value.request_fingerprint_descriptor_bytes(),
        value.base().encode()
    );
    assert!(matches!(
        decode_blob_record(&bytes),
        Ok(BlobRecordV1::ReclaimDescriptorV3(decoded)) if decoded == value
    ));
    assert_ne!(value.custody_digest(), [0; 32]);
    let mut independent = Sha256::new();
    independent.update(b"store.physical.released-drop-custody.v1");
    independent.update(&bytes[BLOB_RECORD_HEADER_BYTES..]);
    assert_eq!(
        value.custody_digest(),
        <[u8; 32]>::from(independent.finalize())
    );
}

#[test]
fn custody_version_hash_and_source_kind_are_fail_closed() {
    let value = BlobReclaimDescriptorV3::new(
        descriptor(BlobReclaimSourceKind::ReleasedGeneration),
        custody(),
    )
    .unwrap();
    assert_eq!(
        BlobReclaimDescriptorV3::new(descriptor(BlobReclaimSourceKind::FailedIngest), custody()),
        Err(BlobRecordDenial::InvalidReclaimDescriptor)
    );
    let mut payload = value.encode()[BLOB_RECORD_HEADER_BYTES..].to_vec();
    payload[V2_PAYLOAD_BYTES] = 2;
    let wrong_version = encode(BlobRecordKind::ReclaimDescriptorV3, &payload).unwrap();
    assert_eq!(
        BlobReclaimDescriptorV3::decode(&wrong_version),
        Err(BlobRecordDenial::InvalidReclaimDescriptor)
    );
    payload[V2_PAYLOAD_BYTES] = CUSTODY_VERSION;
    payload[V2_PAYLOAD_BYTES + 1..V2_PAYLOAD_BYTES + 33].fill(0);
    let zero_source_hash = encode(BlobRecordKind::ReclaimDescriptorV3, &payload).unwrap();
    assert!(BlobReclaimDescriptorV3::decode(&zero_source_hash).is_err());
    let mut invalid_lease = value.encode()[BLOB_RECORD_HEADER_BYTES..].to_vec();
    invalid_lease[PAYLOAD_BYTES - 8..].copy_from_slice(&9_u64.to_le_bytes());
    let invalid_lease = encode(BlobRecordKind::ReclaimDescriptorV3, &invalid_lease).unwrap();
    assert!(BlobReclaimDescriptorV3::decode(&invalid_lease).is_err());
    assert!(BlobReclaimDescriptorV3::decode(&value.encode()[..32]).is_err());
}
