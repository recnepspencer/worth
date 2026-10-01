use super::btree_node::inspect_btree_node_payload;
use crate::integrity_observation::{
    crc32c::crc32c, OfflineIntegrityObservationCounters, OfflineIntegrityOutcome,
    OfflinePhysicalDamageCause, OfflineUnsupportedVersionAxis,
};

fn literal_leaf() -> Vec<u8> {
    // Handwritten v1 frame: one directory cell, key "ab", opaque value "XYZ".
    let mut bytes = vec![0_u8; 117];
    bytes[..8].copy_from_slice(b"WRC11BTN");
    bytes[8] = 1;
    bytes[9] = 1;
    bytes[12..14].copy_from_slice(&4_u16.to_le_bytes());
    bytes[14..16].copy_from_slice(&1_u16.to_le_bytes());
    bytes[16..20].copy_from_slice(&117_u32.to_le_bytes());
    bytes[104..106].copy_from_slice(&112_u16.to_le_bytes());
    bytes[106..108].copy_from_slice(&2_u16.to_le_bytes());
    bytes[108..110].copy_from_slice(&114_u16.to_le_bytes());
    bytes[110..112].copy_from_slice(&3_u16.to_le_bytes());
    bytes[112..114].copy_from_slice(b"ab");
    bytes[114..117].copy_from_slice(b"XYZ");
    reseal(&mut bytes);
    bytes
}

fn reseal(bytes: &mut [u8]) {
    let checksum = crc32c(&[&bytes[..96], &bytes[100..]]);
    bytes[96..100].copy_from_slice(&checksum.to_le_bytes());
}

fn literal_catalog_leaf() -> Vec<u8> {
    let mut bytes = vec![0_u8; 160];
    bytes[..8].copy_from_slice(b"WRC11BTN");
    bytes[8] = 1;
    bytes[9] = 1;
    bytes[12..14].copy_from_slice(&1_u16.to_le_bytes());
    bytes[14..16].copy_from_slice(&1_u16.to_le_bytes());
    bytes[16..20].copy_from_slice(&160_u32.to_le_bytes());
    bytes[104..106].copy_from_slice(&112_u16.to_le_bytes());
    bytes[106..108].copy_from_slice(&24_u16.to_le_bytes());
    bytes[108..110].copy_from_slice(&136_u16.to_le_bytes());
    bytes[110..112].copy_from_slice(&24_u16.to_le_bytes());
    bytes[112..128].fill(2);
    bytes[128..136].copy_from_slice(&1_u64.to_be_bytes());
    bytes[136..152].fill(3);
    bytes[152..160].copy_from_slice(&4_u64.to_le_bytes());
    reseal(&mut bytes);
    bytes
}

#[test]
fn resealed_catalog_cell_with_wrong_width_is_not_intact() {
    let mut bytes = literal_catalog_leaf();
    let mut counters = OfflineIntegrityObservationCounters::default();
    let facts = inspect_btree_node_payload(&bytes, 1, &mut counters).unwrap();
    assert_eq!(facts.keys[0].len(), 24);
    assert_eq!(facts.leaf_records[0].1, 4);

    // Keep a contiguous, checksummed slot but shift one byte from key to
    // value. A generic slotted-node reader would still admit this frame.
    bytes[106..108].copy_from_slice(&23_u16.to_le_bytes());
    bytes[108..110].copy_from_slice(&135_u16.to_le_bytes());
    bytes[110..112].copy_from_slice(&25_u16.to_le_bytes());
    reseal(&mut bytes);
    assert!(matches!(
        inspect_btree_node_payload(&bytes, 1, &mut counters),
        Err(OfflineIntegrityOutcome::Damaged(damage))
            if damage.cause() == OfflinePhysicalDamageCause::MalformedPayload
    ));
}

#[test]
fn independent_literal_reader_accepts_shape_and_detects_checksum_scope_and_directory_damage() {
    let bytes = literal_leaf();
    let mut counters = OfflineIntegrityObservationCounters::default();
    let facts = inspect_btree_node_payload(&bytes, 4, &mut counters).unwrap();
    assert_eq!(facts.level, 0);
    assert_eq!(facts.cell_count, 1);
    assert!(facts.first_child.is_none());
    assert!(facts.separator_children.is_empty());
    assert_eq!(counters.checksum_calculations(), 1);

    let mut flipped = bytes.clone();
    flipped[112] ^= 1;
    assert!(matches!(
        inspect_btree_node_payload(&flipped, 4, &mut counters),
        Err(OfflineIntegrityOutcome::Damaged(damage))
            if damage.cause() == OfflinePhysicalDamageCause::ChecksumMismatch
    ));
    assert!(matches!(
        inspect_btree_node_payload(&bytes, 5, &mut counters),
        Err(OfflineIntegrityOutcome::Damaged(damage))
            if damage.cause() == OfflinePhysicalDamageCause::ScopeMismatch
    ));
    let mut resealed_bad_directory = bytes.clone();
    resealed_bad_directory[104..106].copy_from_slice(&113_u16.to_le_bytes());
    reseal(&mut resealed_bad_directory);
    assert!(matches!(
        inspect_btree_node_payload(&resealed_bad_directory, 4, &mut counters),
        Err(OfflineIntegrityOutcome::Damaged(_))
    ));
}

#[test]
fn unknown_literal_node_version_remains_unsupported() {
    let mut bytes = literal_leaf();
    bytes[8] = 2;
    assert!(matches!(
        inspect_btree_node_payload(&bytes, 4, &mut OfflineIntegrityObservationCounters::default()),
        Err(OfflineIntegrityOutcome::Unsupported(version))
            if version.axis() == OfflineUnsupportedVersionAxis::BTreeNode && version.observed() == 2
    ));
}
