use sha2::{Digest, Sha256};
use worth_store_physical_format::{PersistedRecordIdentity, RecordAnchorName};

use super::{decode_member, AnchoredRecordHeader, HEADER_BYTES};
use crate::physical_runtime::{
    AnchoredRecordDamage, AnchoredRecordMemberLocator, PhysicalRecordId,
    ANCHORED_RECORD_MEMBER_LOCATOR_BYTES, MAXIMUM_ANCHORED_RECORD_MEMBERS,
};

fn anchor(name: &str) -> RecordAnchorName {
    RecordAnchorName::new(name).unwrap()
}

fn header_bytes(record: &[u8]) -> [u8; HEADER_BYTES] {
    record[..HEADER_BYTES].try_into().unwrap()
}

fn locator(ordinal: u64, content: &[u8]) -> AnchoredRecordMemberLocator {
    AnchoredRecordMemberLocator::new(
        PhysicalRecordId::from_persisted(PersistedRecordIdentity::new([7; 16], ordinal).unwrap()),
        Sha256::digest(content).into(),
        content.len() as u64,
    )
}

#[test]
fn a_record_decodes_only_under_its_own_anchor_and_generation() {
    let payload = b"checkpoint image".to_vec();
    let header = AnchoredRecordHeader::describe(anchor("relational.checkpoint"), 3, &payload, 0);
    let record = header.encode_record(&[], &payload);
    assert_eq!(record.len(), HEADER_BYTES + payload.len());
    assert_eq!(record.len() as u64, header.record_bytes());
    assert_eq!(&record[HEADER_BYTES..], payload.as_slice());

    let decoded =
        AnchoredRecordHeader::decode(&header_bytes(&record), anchor("relational.checkpoint"), 3)
            .unwrap();
    assert_eq!(decoded, header);
    assert_eq!(
        decoded.verify_payload_digest(Sha256::digest(&payload).into()),
        Ok(())
    );
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
fn the_member_table_sits_between_the_header_and_the_payload_in_order() {
    let payload = vec![0x5a; 1_000];
    let members = [locator(9, b"region a"), locator(3, b"symbol page")];
    let header = AnchoredRecordHeader::describe(anchor("relational.home"), 1, &payload, 2);
    let record = header.encode_record(&members, &payload);
    assert_eq!(record.len() as u64, header.record_bytes());
    let table = &record[HEADER_BYTES..HEADER_BYTES + 2 * ANCHORED_RECORD_MEMBER_LOCATOR_BYTES];
    for (entry, expected) in table
        .chunks_exact(ANCHORED_RECORD_MEMBER_LOCATOR_BYTES)
        .zip(&members)
    {
        assert_eq!(decode_member(entry.try_into().unwrap()), Ok(*expected));
    }
    assert_eq!(&record[record.len() - payload.len()..], payload.as_slice());
    let next = AnchoredRecordHeader::describe(anchor("relational.home"), 1, &payload, 1);
    assert_ne!(
        Sha256::digest(next.encode_record(&members[..1], &payload)),
        Sha256::digest(&record)
    );
}

#[test]
fn a_damaged_frame_or_table_refuses() {
    let payload = vec![7; 64];
    let header = AnchoredRecordHeader::describe(anchor("relational.home"), 1, &payload, 0);
    let mut record = header.encode_record(&[], &payload);
    assert_eq!(
        header.verify_payload_digest([0; 32]),
        Err(AnchoredRecordDamage::PayloadDigestMismatch)
    );

    let mut oversized = header;
    oversized.member_count = MAXIMUM_ANCHORED_RECORD_MEMBERS as u64 + 1;
    assert_eq!(
        AnchoredRecordHeader::decode(&oversized.encode(), anchor("relational.home"), 1),
        Err(AnchoredRecordDamage::MalformedMemberTable)
    );
    assert_eq!(
        decode_member(&[0; ANCHORED_RECORD_MEMBER_LOCATOR_BYTES]),
        Err(AnchoredRecordDamage::MalformedMemberTable)
    );

    record[0] ^= 1;
    assert_eq!(
        AnchoredRecordHeader::decode(&header_bytes(&record), anchor("relational.home"), 1),
        Err(AnchoredRecordDamage::WrongMagic)
    );
}
