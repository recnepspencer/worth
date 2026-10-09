use super::*;

fn record(ordinal: u64) -> [u8; 24] {
    let mut id = [4; 24];
    id[16..].copy_from_slice(&ordinal.to_le_bytes());
    id
}

fn released_manifest() -> Vec<u8> {
    let publication = record(1);
    let chunk = record(2);
    let mut publication_payload = Vec::new();
    publication_payload.extend_from_slice(&[1; 16]);
    publication_payload.extend_from_slice(&[8; 16]);
    publication_payload.extend_from_slice(&[3; 16]);
    publication_payload.extend_from_slice(&7_u64.to_le_bytes());
    publication_payload.extend_from_slice(&record(3));
    publication_payload.extend_from_slice(&[7; 32]);
    publication_payload.extend_from_slice(&4096_u64.to_le_bytes());
    publication_payload.extend_from_slice(&[9; 32]);
    publication_payload.extend_from_slice(&(64_u32 << 10).to_le_bytes());
    publication_payload.extend_from_slice(&[10; 32]);
    assert_eq!(publication_payload.len(), 188);
    let mut publication_frame = Vec::new();
    publication_frame.extend_from_slice(b"WRC11BLB");
    publication_frame.extend_from_slice(&[4, 1, 0, 0]);
    publication_frame.extend_from_slice(&188_u32.to_le_bytes());
    publication_frame.extend_from_slice(&[0; 32]);
    publication_frame.extend_from_slice(&publication_payload);
    let mut envelope = Sha256::new();
    envelope.update(&publication_frame[..16]);
    envelope.update(&publication_payload);
    publication_frame[16..48].copy_from_slice(&envelope.finish());
    let publication_digest = super::super::super::super::sha256::sha256(&publication_frame);
    let mut source = Vec::new();
    source.extend_from_slice(&publication);
    source.extend_from_slice(&publication_digest);
    source.extend_from_slice(&[6; 32]);
    source.extend_from_slice(&publication_frame);
    assert_eq!(source.len(), 324);
    let mut payload = Vec::new();
    payload.extend_from_slice(&[1; 16]);
    payload.extend_from_slice(&[2; 16]);
    payload.push(2);
    payload.extend_from_slice(&324_u16.to_le_bytes());
    payload.extend_from_slice(&source);
    payload.extend_from_slice(&2_u16.to_le_bytes());
    let mut dropped = Sha256::new();
    dropped.update(DROP_DOMAIN);
    dropped.update(&2_u16.to_le_bytes());
    dropped.update(&publication);
    dropped.update(&chunk);
    payload.extend_from_slice(&dropped.finish());
    payload.extend_from_slice(&publication);
    payload.extend_from_slice(&chunk);
    payload.extend_from_slice(&8_u64.to_le_bytes());
    payload.push(1);
    payload
}

#[test]
fn released_source_is_distinct_and_embedded_publication_is_authenticated() {
    let mut payload = released_manifest();
    let fact = decode_manifest_v3(&payload, [9; 32]).expect("independent release manifest");
    let BlobFact::ReleasedDropSetManifest {
        object,
        generation,
        publication_record,
        publication_digest,
        issuer_evidence_digest,
        basis_digest,
        dropped,
        ..
    } = fact
    else {
        panic!("released source kind");
    };
    assert_eq!(
        (object, generation, publication_record),
        ([3; 16], 7, record(1))
    );
    assert_eq!(issuer_evidence_digest, [6; 32]);
    assert_eq!(
        publication_digest,
        super::super::super::super::sha256::sha256(&payload[123..359])
    );
    assert_eq!(dropped, vec![record(1), record(2)]);
    let mut expected = Sha256::new();
    expected.update(RELEASE_DOMAIN);
    expected.update(&[1; 16]);
    expected.update(&[2]);
    expected.update(&payload[35..359]);
    assert_eq!(basis_digest, expected.finish());

    // Inner publication bytes cannot diverge from the immutable source digest.
    payload[123] ^= 1;
    assert!(decode_manifest_v3(&payload, [9; 32]).is_none());
}

#[test]
fn versioned_descriptor_carries_release_kind_and_rejects_old_width() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&[1; 16]);
    payload.extend_from_slice(&[2; 16]);
    payload.push(2);
    payload.extend_from_slice(&[3; 32]);
    payload.extend_from_slice(&record(4));
    payload.extend_from_slice(&[5; 32]);
    payload.extend_from_slice(&2_u16.to_le_bytes());
    payload.extend_from_slice(&9_u64.to_le_bytes());
    payload.extend_from_slice(&10_u64.to_le_bytes());
    payload.extend_from_slice(&[0; 57]);
    payload.extend_from_slice(&2_u64.to_le_bytes());
    payload.push(1);
    let mut custody_payload = payload.clone();
    assert!(matches!(
        decode_descriptor_v2(&payload, [11; 32]),
        Some(BlobFact::ReleasedReclaimDescriptor {
            source_root: 9,
            candidate_root: 10,
            ..
        })
    ));
    payload[32] = 1;
    assert!(decode_descriptor_v2(&payload, [11; 32]).is_none());
    payload.pop();
    assert!(decode_descriptor_v2(&payload, [11; 32]).is_none());

    custody_payload.push(1);
    for code in 1..=8 {
        custody_payload.extend_from_slice(&[code; 32]);
    }
    custody_payload.extend_from_slice(&1_u64.to_le_bytes());
    custody_payload.extend_from_slice(&2_u64.to_le_bytes());
    let Some(BlobFact::ReleasedReclaimDescriptor {
        custody: Some(custody),
        ..
    }) = decode_descriptor_v3(&custody_payload, [12; 32])
    else {
        panic!("released custody descriptor");
    };
    assert_eq!(custody.source_root_sha256, [1; 32]);
    assert_eq!(custody.source_free_space_sha256, [2; 32]);
    assert_eq!(custody.idempotency, [7; 32]);
    assert_eq!(custody.fingerprint, [8; 32]);
    custody_payload[205] = 2;
    assert!(decode_descriptor_v3(&custody_payload, [12; 32]).is_none());
}
