use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV2, BlobReclaimDescriptorV3, BlobReclaimSourceKind,
    BlobRecordV1, OriginalDropReservationRequestV1, PersistedRecordIdentity, ReleasedDropCustodyV1,
};

use super::release_needs_certificate;

fn descriptor(kind: BlobReclaimSourceKind) -> BlobReclaimDescriptorV2 {
    BlobReclaimDescriptorV2::new(
        [1; 16],
        [2; 16],
        kind,
        [3; 32],
        PersistedRecordIdentity::new([4; 16], 1).unwrap(),
        [5; 32],
        1,
        7,
        8,
        None,
        1,
        true,
    )
    .unwrap()
}

#[test]
fn covered_selected_v2_and_v3_release_require_certificate() {
    let released = descriptor(BlobReclaimSourceKind::ReleasedGeneration);
    let failed = descriptor(BlobReclaimSourceKind::FailedIngest);
    let request = OriginalDropReservationRequestV1::new([12; 32], [13; 32], 1, 2).unwrap();
    let custody = ReleasedDropCustodyV1::new(
        [6; 32], [7; 32], [8; 32], [9; 32], [10; 32], [11; 32], request,
    )
    .unwrap();
    let certified = BlobReclaimDescriptorV3::new(released, custody).unwrap();
    for encoded in [released.encode(), certified.encode()] {
        let decoded = decode_blob_record(&encoded).unwrap();
        let released = matches!(
            &decoded,
            BlobRecordV1::ReclaimDescriptorV2(value)
                if value.source_kind() == BlobReclaimSourceKind::ReleasedGeneration
        ) || matches!(&decoded, BlobRecordV1::ReclaimDescriptorV3(_));
        assert!(release_needs_certificate(released, false));
        assert!(!release_needs_certificate(released, true));
    }
    let failed_frame = failed.encode();
    let decoded = decode_blob_record(&failed_frame).unwrap();
    let released = matches!(
        decoded,
        BlobRecordV1::ReclaimDescriptorV2(value)
            if value.source_kind() == BlobReclaimSourceKind::ReleasedGeneration
    );
    assert!(!release_needs_certificate(released, false));
}
