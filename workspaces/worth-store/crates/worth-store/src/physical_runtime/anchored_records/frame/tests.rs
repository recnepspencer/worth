use sha2::{Digest, Sha256};
use worth_store_physical_format::RecordAnchorName;

use super::{AnchoredRecordHeader, HEADER_BYTES};
use crate::physical_runtime::AnchoredRecordDamage;

fn anchor(name: &str) -> RecordAnchorName {
    RecordAnchorName::new(name).unwrap()
}

fn header_bytes(record: &[u8]) -> [u8; HEADER_BYTES] {
    record[..HEADER_BYTES].try_into().unwrap()
}

#[test]
fn a_record_decodes_only_under_its_own_anchor_and_generation() {
    let payload = b"checkpoint image".to_vec();
    let header = AnchoredRecordHeader::describe(anchor("relational.checkpoint"), 3, &payload);
    let record = header.encode_record(&payload);
    assert_eq!(record.len(), HEADER_BYTES + payload.len());
    assert_eq!(&record[HEADER_BYTES..], payload.as_slice());

    let decoded =
        AnchoredRecordHeader::decode(&header_bytes(&record), anchor("relational.checkpoint"), 3)
            .unwrap();
    assert_eq!(decoded, header);
    assert_eq!(decoded.verify_payload(&payload), Ok(()));
    assert_eq!(
        AnchoredRecordHeader::decode(&header_bytes(&record), anchor("relational.home"), 3),
        Err(AnchoredRecordDamage::WrongAnchor)
    );
    assert_eq!(
        AnchoredRecordHeader::decode(&header_bytes(&record), anchor("relational.checkpoint"), 2),
        Err(AnchoredRecordDamage::WrongGeneration)
    );
}

#[test]
fn the_record_digest_covers_the_frame_and_the_payload() {
    let payload = vec![0x5a; 1_000];
    let header = AnchoredRecordHeader::describe(anchor("relational.home"), 1, &payload);
    let digest: [u8; 32] = Sha256::digest(header.encode_record(&payload)).into();
    assert_eq!(header.record_sha256(&payload), digest);
    let next = AnchoredRecordHeader::describe(anchor("relational.home"), 2, &payload);
    assert_ne!(next.record_sha256(&payload), digest);
}

#[test]
fn a_damaged_frame_or_payload_refuses() {
    let payload = vec![7; 64];
    let header = AnchoredRecordHeader::describe(anchor("relational.home"), 1, &payload);
    let mut record = header.encode_record(&payload);

    let mut flipped = payload.clone();
    flipped[10] ^= 1;
    assert_eq!(
        header.verify_payload(&flipped),
        Err(AnchoredRecordDamage::PayloadDigestMismatch)
    );
    assert_eq!(
        header.verify_payload(&payload[..63]),
        Err(AnchoredRecordDamage::Truncated)
    );

    record[0] ^= 1;
    assert_eq!(
        AnchoredRecordHeader::decode(&header_bytes(&record), anchor("relational.home"), 1),
        Err(AnchoredRecordDamage::WrongMagic)
    );
}
