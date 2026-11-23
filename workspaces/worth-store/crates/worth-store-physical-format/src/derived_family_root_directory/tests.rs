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
fn quarantine_marker_roundtrips_and_retired_v1_directory_is_rejected() {
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
    assert_eq!(
        DerivedFamilyRootDirectoryV1::decode(&old),
        Err(DerivedFamilyDirectoryDenial::UnsupportedVersion)
    );
}

#[test]
fn validate_header_matches_decode_on_compound_corruption() {
    let build_raw = |entries: &[(u16, u64)]| {
        let mut bytes = DerivedFamilyRootDirectoryV1::new(vec![]).unwrap().encode();
        bytes[9] = entries.len() as u8;
        for &(tag, ordinal) in entries {
            bytes.extend_from_slice(&tag.to_le_bytes());
            bytes.extend_from_slice(&[7; 16]);
            bytes.extend_from_slice(&ordinal.to_le_bytes());
        }
        bytes
    };

    // Duplicate tag followed by unknown family tag: owned decode validates all entries first.
    let dup_then_unknown = build_raw(&[(1, 11), (1, 12), (3, 13)]);
    assert_eq!(
        DerivedFamilyRootDirectoryV1::validate_header(&dup_then_unknown),
        Err(DerivedFamilyDirectoryDenial::UnknownFamily)
    );
    assert_eq!(
        DerivedFamilyRootDirectoryV1::decode(&dup_then_unknown),
        Err(DerivedFamilyDirectoryDenial::UnknownFamily)
    );

    // Duplicate tag followed by invalid record identity (ordinal 0): must return InvalidRecordIdentity.
    let dup_then_bad_id = build_raw(&[(1, 11), (1, 12), (2, 0)]);
    assert_eq!(
        DerivedFamilyRootDirectoryV1::validate_header(&dup_then_bad_id),
        Err(DerivedFamilyDirectoryDenial::InvalidRecordIdentity)
    );
    assert_eq!(
        DerivedFamilyRootDirectoryV1::decode(&dup_then_bad_id),
        Err(DerivedFamilyDirectoryDenial::InvalidRecordIdentity)
    );

    // Valid entries in non-canonical order: both return NonCanonicalOrder.
    for raw in [build_raw(&[(1, 11), (1, 12)]), build_raw(&[(2, 11), (1, 12)])] {
        assert_eq!(
            DerivedFamilyRootDirectoryV1::validate_header(&raw),
            Err(DerivedFamilyDirectoryDenial::NonCanonicalOrder)
        );
        assert_eq!(
            DerivedFamilyRootDirectoryV1::decode(&raw),
            Err(DerivedFamilyDirectoryDenial::NonCanonicalOrder)
        );
    }
}
