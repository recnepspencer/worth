use sha2::{Digest, Sha256};

use super::*;
use crate::{BlobRecordDenial, BlobRecordKind, PersistedRecordIdentity, BLOB_RECORD_HEADER_BYTES};

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).expect("valid record")
}

fn basis() -> FailedIngestReclaimBasisV1 {
    FailedIngestReclaimBasisV1::new([2; 16], record(1), [3; 32], record(2), [4; 32])
        .expect("valid selected-source shape")
}

#[test]
fn versioned_never_reserved_manifest_and_exact_reserved_transition_round_trip() {
    let manifest = DropSetManifestV2::new([5; 16], [6; 16], basis(), vec![record(3)], 7).unwrap();
    let manifest_bytes = manifest.encode();
    assert_eq!(
        DropSetManifestV2::decode(&manifest_bytes),
        Ok(manifest.clone())
    );
    assert!(matches!(crate::decode_blob_record(&manifest_bytes),
        Ok(crate::BlobRecordV1::DropSetManifestV2(value)) if value == manifest));
    assert_eq!(manifest.drop_set().dropped(), &[record(3)]);
    let request = OriginalDropReservationRequestV1::new([8; 32], [9; 32], 0, 4).unwrap();
    let reserved = OriginalDropReservedV1::new(
        manifest.store(),
        manifest.reclaim_attempt(),
        record(4),
        Sha256::digest(&manifest_bytes).into(),
        manifest.source_basis_digest(),
        manifest.never_reserved_slot_generation(),
        8,
        request,
    )
    .unwrap();
    assert_eq!(
        OriginalDropReservedV1::decode(&reserved.encode()),
        Ok(reserved)
    );
    assert!(matches!(crate::decode_blob_record(&reserved.encode()),
        Ok(crate::BlobRecordV1::OriginalDropReserved(value)) if value == reserved));
    assert_eq!(reserved.request().lease_issuance_generation(), 0);
}

#[test]
fn versioned_manifest_and_reservation_reject_corrupt_or_nonmonotonic_state() {
    let manifest = DropSetManifestV2::new([5; 16], [6; 16], basis(), vec![record(3)], 7).unwrap();
    let mut bytes = manifest.encode();
    let last = bytes.len() - 1;
    bytes[last] = 2;
    assert!(DropSetManifestV2::decode(&bytes).is_err());
    assert!(DropSetManifestV2::new([5; 16], [6; 16], basis(), vec![record(3)], 0).is_err());
    let request = OriginalDropReservationRequestV1::new([8; 32], [9; 32], 0, 4).unwrap();
    assert!(OriginalDropReservedV1::new(
        [5; 16],
        [6; 16],
        record(4),
        [7; 32],
        [8; 32],
        7,
        9,
        request,
    )
    .is_ok());
    assert!(OriginalDropReservedV1::new(
        [5; 16],
        [6; 16],
        record(4),
        [7; 32],
        [8; 32],
        7,
        7,
        request,
    )
    .is_err());
    assert!(OriginalDropReservationRequestV1::new([8; 32], [9; 32], 0, 0).is_err());
}

#[test]
fn maximum_sorted_drop_set_round_trips_without_granting_drop_authority() {
    let dropped = (3..=1026).map(record).collect::<Vec<_>>();
    let manifest = DropSetManifestV1::new([5; 16], [6; 16], basis(), dropped.clone())
        .expect("maximum bounded manifest");
    let bytes = manifest.encode();
    assert!(bytes.len() < 64 << 10);
    assert_eq!(DropSetManifestV1::decode(&bytes), Ok(manifest.clone()));
    assert_eq!(manifest.count(), 1024);
    assert_eq!(manifest.dropped(), dropped);

    let descriptor = BlobReclaimDescriptorV1::new(
        manifest.store(),
        manifest.reclaim_attempt(),
        manifest.source_basis_digest(),
        record(1027),
        Sha256::digest(&bytes).into(),
        manifest.count(),
        7,
        8,
    )
    .expect("bounded descriptor shape");
    let descriptor_bytes = descriptor.encode();
    assert!(descriptor_bytes.len() < 1024);
    assert_eq!(
        BlobReclaimDescriptorV1::decode(&descriptor_bytes),
        Ok(descriptor)
    );
    assert_eq!(descriptor.manifest_count(), manifest.count());
}

#[test]
fn malformed_or_unbounded_id_inventory_is_denied_before_publication() {
    for dropped in [
        vec![],
        vec![record(3), record(3)],
        vec![record(4), record(3)],
        vec![record(1)],
        vec![record(2)],
        (3..=1027).map(record).collect(),
    ] {
        assert_eq!(
            DropSetManifestV1::new([5; 16], [6; 16], basis(), dropped),
            Err(BlobRecordDenial::InvalidDropSet),
        );
    }
    let manifest =
        DropSetManifestV1::new([5; 16], [6; 16], basis(), vec![record(3)]).expect("valid manifest");
    let bytes = manifest.encode();
    let mut payload = bytes[BLOB_RECORD_HEADER_BYTES..].to_vec();
    payload[194] ^= 1;
    assert_eq!(
        DropSetManifestV1::decode_payload(&payload),
        Err(BlobRecordDenial::InvalidDropSet),
    );
    assert_eq!(
        BlobReclaimDescriptorV1::new(
            [5; 16],
            [6; 16],
            basis().digest([5; 16]),
            record(4),
            [7; 32],
            1,
            7,
            9
        ),
        Err(BlobRecordDenial::InvalidReclaimDescriptor),
    );
}

#[test]
fn borrowed_failed_ingest_manifests_match_owned_current_wire_facts() {
    let dropped = vec![record(3), record(4)];
    let v2 = DropSetManifestV2::new([5; 16], [6; 16], basis(), dropped, 7).unwrap();
    let v2_bytes = v2.encode();
    let borrowed_v2 = DropSetManifestV2View::decode(&v2_bytes).unwrap();
    assert_eq!(borrowed_v2.store(), v2.store());
    assert_eq!(borrowed_v2.reclaim_attempt(), v2.reclaim_attempt());
    assert_eq!(borrowed_v2.source_basis(), v2.source_basis());
    assert_eq!(borrowed_v2.source_basis_digest(), v2.source_basis_digest());
    assert_eq!(borrowed_v2.count(), v2.count());
    assert_eq!(borrowed_v2.dropped_digest(), v2.drop_set().dropped_digest());
    assert_eq!(borrowed_v2.never_reserved_slot_generation(), 7);
    assert!(borrowed_v2.contains_record(record(4)));
    assert!(!borrowed_v2.contains_record(record(5)));
    assert_eq!(
        borrowed_v2.canonical_frame_sha256(),
        <[u8; 32]>::from(Sha256::digest(&v2_bytes))
    );
    let retired = v2.drop_set().encode();
    assert_eq!(
        DropSetManifestV2View::decode(&retired).unwrap_err(),
        BlobRecordDenial::UnknownKind
    );
    assert_eq!(DropSetManifestV2::decode(&v2_bytes), Ok(v2));
}

#[test]
fn borrowed_failed_ingest_manifests_deny_malformed_current_grammar() {
    let manifest =
        DropSetManifestV2::new([5; 16], [6; 16], basis(), vec![record(3), record(4)], 7).unwrap();
    let payload = manifest.encode()[BLOB_RECORD_HEADER_BYTES..].to_vec();
    let mut malformed = Vec::new();
    let mut zero_count = payload.clone();
    zero_count[160..162].copy_from_slice(&0_u16.to_le_bytes());
    malformed.push(zero_count);
    let mut wrong_count = payload.clone();
    wrong_count[160..162].copy_from_slice(&3_u16.to_le_bytes());
    malformed.push(wrong_count);
    let mut unordered = payload.clone();
    unordered[194..218].copy_from_slice(&payload[218..242]);
    unordered[218..242].copy_from_slice(&payload[194..218]);
    malformed.push(unordered);
    let mut wrong_digest = payload.clone();
    wrong_digest[162] ^= 1;
    malformed.push(wrong_digest);
    let mut excluded_source = payload.clone();
    excluded_source[48..72].copy_from_slice(&payload[194..218]);
    malformed.push(excluded_source);
    let mut forged_source = payload.clone();
    forged_source[32..48].fill(0);
    malformed.push(forged_source);
    for payload in malformed {
        let bytes = super::super::envelope::encode(BlobRecordKind::DropSetManifestV2, &payload)
            .expect("bounded malformed frame");
        let borrowed = DropSetManifestV2View::decode(&bytes).unwrap_err();
        let owned = DropSetManifestV2::decode(&bytes).unwrap_err();
        assert_eq!(borrowed, owned);
    }

    let v2 = DropSetManifestV2::new([5; 16], [6; 16], basis(), vec![record(3)], 7).unwrap();
    let payload = v2.encode()[BLOB_RECORD_HEADER_BYTES..].to_vec();
    for trailer in [0_u64.to_le_bytes().to_vec(), 7_u64.to_le_bytes().to_vec()] {
        let mut malformed = payload.clone();
        let trailer_start = malformed.len() - 9;
        malformed[trailer_start..trailer_start + 8].copy_from_slice(&trailer);
        if trailer[0] == 7 {
            *malformed.last_mut().unwrap() = 2;
        }
        let bytes = super::super::envelope::encode(BlobRecordKind::DropSetManifestV2, &malformed)
            .expect("bounded malformed frame");
        assert_eq!(
            DropSetManifestV2View::decode(&bytes).unwrap_err(),
            DropSetManifestV2::decode(&bytes).unwrap_err()
        );
    }
}
