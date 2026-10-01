use super::*;

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([7; 16], ordinal).unwrap()
}

#[test]
fn directory_round_trip_and_stable_family_tags() {
    let directory = DerivedFamilyRootDirectoryV1::new(vec![
        DerivedFamilyRootEntry::new(DurableArtifactFamilyId::BlobCatalog, record(11)).unwrap(),
        DerivedFamilyRootEntry::new(DurableArtifactFamilyId::DedupeIndex, record(12)).unwrap(),
    ])
    .unwrap();
    let bytes = directory.encode();
    assert_eq!(&bytes[..8], b"WRC11IDX");
    assert_eq!(bytes[8], 2);
    assert_eq!(&bytes[101..103], &1_u16.to_le_bytes());
    assert_eq!(&bytes[127..129], &2_u16.to_le_bytes());
    assert_eq!(DerivedFamilyRootDirectoryV1::decode(&bytes), Ok(directory));
}

#[test]
fn directory_rejects_unknown_duplicate_and_noncanonical_entries() {
    let first =
        DerivedFamilyRootEntry::new(DurableArtifactFamilyId::BlobCatalog, record(11)).unwrap();
    let second =
        DerivedFamilyRootEntry::new(DurableArtifactFamilyId::DedupeIndex, record(12)).unwrap();
    assert_eq!(
        DerivedFamilyRootDirectoryV1::new(vec![first, first]),
        Err(DerivedFamilyDirectoryDenial::NonCanonicalOrder),
    );
    assert_eq!(
        DerivedFamilyRootDirectoryV1::new(vec![second, first]),
        Err(DerivedFamilyDirectoryDenial::NonCanonicalOrder),
    );
    let mut bytes = DerivedFamilyRootDirectoryV1::new(vec![first])
        .unwrap()
        .encode();
    bytes[101..103].copy_from_slice(&3_u16.to_le_bytes());
    assert_eq!(
        DerivedFamilyRootDirectoryV1::decode(&bytes),
        Err(DerivedFamilyDirectoryDenial::UnknownFamily),
    );
}

#[test]
fn directory_rejects_trailing_bytes_and_reserved_field() {
    let mut bytes = DerivedFamilyRootDirectoryV1::new(vec![]).unwrap().encode();
    bytes.push(0);
    assert_eq!(
        DerivedFamilyRootDirectoryV1::decode(&bytes),
        Err(DerivedFamilyDirectoryDenial::InvalidLength),
    );
    let mut bytes = DerivedFamilyRootDirectoryV1::new(vec![]).unwrap().encode();
    bytes[11] = 1;
    assert_eq!(
        DerivedFamilyRootDirectoryV1::decode(&bytes),
        Err(DerivedFamilyDirectoryDenial::ReservedFieldNonZero),
    );
}

#[test]
fn indexed_through_marker_round_trips_and_absence_is_canonical() {
    let marker = IndexedThroughBlobPublication::new(7, record(19), [3; 32]).unwrap();
    let directory = DerivedFamilyRootDirectoryV1::new(vec![])
        .unwrap()
        .with_indexed_through(marker);
    let bytes = directory.encode();
    assert_eq!(bytes[10], 1);
    assert_eq!(DerivedFamilyRootDirectoryV1::decode(&bytes), Ok(directory));
    let mut absent = DerivedFamilyRootDirectoryV1::new(vec![]).unwrap().encode();
    absent[12] = 1;
    assert_eq!(
        DerivedFamilyRootDirectoryV1::decode(&absent),
        Err(DerivedFamilyDirectoryDenial::ReservedFieldNonZero),
    );
}

#[test]
fn quarantine_marker_roundtrips_and_v1_directory_remains_readable() {
    let directory = DerivedFamilyRootDirectoryV1::new(vec![])
        .unwrap()
        .with_indexed_through_quarantine(Some(record(31)));
    assert_eq!(
        DerivedFamilyRootDirectoryV1::decode(&directory.encode()),
        Ok(directory)
    );
    let mut old = DerivedFamilyRootDirectoryV1::new(vec![]).unwrap().encode();
    old[8] = 1;
    old.drain(76..101);
    let decoded = DerivedFamilyRootDirectoryV1::decode(&old).unwrap();
    assert_eq!(decoded.indexed_through_quarantine(), None);
}
