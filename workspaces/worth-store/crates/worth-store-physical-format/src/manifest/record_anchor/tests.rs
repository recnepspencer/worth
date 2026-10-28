use super::*;

fn name(value: &str) -> RecordAnchorName {
    RecordAnchorName::new(value).unwrap()
}

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([7; 16], ordinal).unwrap()
}

fn advance(anchor: &str, expected: u64, ordinal: u64) -> RecordAnchorAdvance {
    RecordAnchorAdvance::new(name(anchor), expected, record(ordinal), [ordinal as u8; 32]).unwrap()
}

#[test]
fn names_admit_only_bounded_lowercase_ascii() {
    assert_eq!(
        name("relational.canonical-commits").as_str(),
        "relational.canonical-commits"
    );
    assert!(RecordAnchorName::new("").is_none());
    assert!(RecordAnchorName::new("Upper").is_none());
    assert!(RecordAnchorName::new("space name").is_none());
    assert!(RecordAnchorName::new(&"a".repeat(33)).is_none());
    assert!(RecordAnchorName::new(&"a".repeat(32)).is_some());
}

#[test]
fn advances_are_sequential_compare_and_set() {
    let table = RecordAnchorTable::EMPTY
        .advanced(advance("journal", 0, 1))
        .unwrap();
    let anchor = table.get(name("journal")).unwrap();
    assert_eq!((anchor.generation(), anchor.record()), (1, record(1)));

    let table = table.advanced(advance("journal", 1, 2)).unwrap();
    assert_eq!(table.generation(name("journal")), 2);
    assert_eq!(
        table.advanced(advance("journal", 1, 3)),
        Err(RecordAnchorAdvanceDenial::GenerationConflict {
            current: 2,
            expected: 1
        })
    );
    assert_eq!(
        RecordAnchorTable::EMPTY.advanced(advance("journal", 4, 1)),
        Err(RecordAnchorAdvanceDenial::GenerationConflict {
            current: 0,
            expected: 4
        })
    );
}

#[test]
fn tables_stay_sorted_and_bounded() {
    let mut table = RecordAnchorTable::EMPTY;
    for anchor in ["d", "b", "c", "a"] {
        table = table.advanced(advance(anchor, 0, 1)).unwrap();
    }
    let names: Vec<_> = table
        .iter()
        .map(|anchor| anchor.name().as_str().to_owned())
        .collect();
    assert_eq!(names, ["a", "b", "c", "d"]);
    assert_eq!(
        table.advanced(advance("e", 0, 1)),
        Err(RecordAnchorAdvanceDenial::TableFull)
    );
    assert!(table.advanced(advance("c", 1, 2)).is_ok());
}

#[test]
fn encoding_round_trips_and_refuses_noncanonical_bytes() {
    let table = RecordAnchorTable::EMPTY
        .advanced(advance("b", 0, 2))
        .unwrap()
        .advanced(advance("a", 0, 1))
        .unwrap();
    let mut bytes = vec![0; RECORD_ANCHOR_TABLE_BYTES];
    table.encode(&mut bytes);
    assert_eq!(RecordAnchorTable::decode(&bytes), Some(table));

    let mut empty = vec![0; RECORD_ANCHOR_TABLE_BYTES];
    RecordAnchorTable::EMPTY.encode(&mut empty);
    assert_eq!(RecordAnchorTable::decode(&empty), None);

    let mut reserved = bytes.clone();
    reserved[3] = 1;
    assert_eq!(RecordAnchorTable::decode(&reserved), None);

    let mut trailing = bytes.clone();
    trailing[8 + 2 * RECORD_ANCHOR_ENTRY_BYTES] = 1;
    assert_eq!(RecordAnchorTable::decode(&trailing), None);

    let mut unsorted = bytes.clone();
    let (first, second) =
        unsorted[8..8 + 2 * RECORD_ANCHOR_ENTRY_BYTES].split_at_mut(RECORD_ANCHOR_ENTRY_BYTES);
    first.swap_with_slice(second);
    assert_eq!(RecordAnchorTable::decode(&unsorted), None);

    let mut zero_generation = bytes.clone();
    zero_generation[8 + 88..8 + 96].fill(0);
    assert_eq!(RecordAnchorTable::decode(&zero_generation), None);

    let mut bad_name = bytes;
    bad_name[8] = b'A';
    assert_eq!(RecordAnchorTable::decode(&bad_name), None);
}

mod root_layout {
    use super::*;
    use crate::{
        DurablePhysicalRootManifest, ManifestBlockReference, PhysicalRecordFormatDeclaration,
        RootManifestDenial,
    };

    fn anchored_root(anchors: RecordAnchorTable) -> Option<DurablePhysicalRootManifest> {
        let route = ManifestBlockReference::new(1, 1, 0, 31, record(1), record(2)).unwrap();
        DurablePhysicalRootManifest::builder(1, 9, 2, 43)
            .record_count(2)
            .next_block(2)
            .routing_root(Some(route))
            .record_anchors(anchors)
            .admit()
    }

    #[test]
    fn anchored_roots_use_schema_eleven_and_round_trip() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let anchors = RecordAnchorTable::EMPTY
            .advanced(advance("journal", 0, 2))
            .unwrap();
        let root = anchored_root(anchors).unwrap();
        let bytes = root.encode(format);
        assert_eq!(bytes.len(), root.encoded_frame_bytes());
        assert_eq!(bytes[9], 11);
        let (decoded, _) = DurablePhysicalRootManifest::decode(&bytes, u16::MAX).unwrap();
        assert_eq!(decoded, root);
        assert_eq!(decoded.record_anchors(), anchors);

        let plain = anchored_root(RecordAnchorTable::EMPTY).unwrap();
        assert_ne!(plain.encode(format)[9], 11);
        assert!(DurablePhysicalRootManifest::maximum_encoding_scratch_bytes() >= bytes.len());
    }

    #[test]
    fn anchors_must_name_records_the_root_routes() {
        let outside = RecordAnchorTable::EMPTY
            .advanced(advance("journal", 0, 99))
            .unwrap();
        assert!(anchored_root(outside).is_none());
    }

    #[test]
    fn schema_eleven_refuses_an_empty_or_damaged_table() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let anchors = RecordAnchorTable::EMPTY
            .advanced(advance("journal", 0, 2))
            .unwrap();
        let bytes = anchored_root(anchors).unwrap().encode(format);
        let table = bytes.len() - RECORD_ANCHOR_TABLE_BYTES;
        let mut emptied = bytes.clone();
        emptied[table..].fill(0);
        let mut damaged = bytes;
        damaged[table + 8] = b'Z';
        for altered in [emptied, damaged] {
            assert!(matches!(
                DurablePhysicalRootManifest::decode(&altered, u16::MAX),
                Err(RootManifestDenial::Frame(_) | RootManifestDenial::InvalidRecordIdentity)
            ));
        }
    }
}
