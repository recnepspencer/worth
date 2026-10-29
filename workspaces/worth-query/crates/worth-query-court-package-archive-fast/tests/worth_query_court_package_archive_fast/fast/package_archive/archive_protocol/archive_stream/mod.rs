pub(crate) use worth_query_worlds::facade::package_archive::archive_protocol::archive_stream::*;

pub(crate) mod predecessor;

#[test]
fn version_two_minimal_archive_is_deterministic_and_frozen() {
    for (version, vector) in [
        (6_u16, RETAINED_VERSION_TWO_MINIMAL_ARCHIVE_HEX),
        (7_u16, RETAINED_RECORD_SEVEN_MINIMAL_ARCHIVE_HEX),
        (8_u16, RETAINED_RECORD_EIGHT_MINIMAL_ARCHIVE_HEX),
        (9_u16, RETAINED_RECORD_NINE_MINIMAL_ARCHIVE_HEX),
        (10_u16, RETAINED_RECORD_TEN_MINIMAL_ARCHIVE_HEX),
        (12_u16, RETAINED_RECORD_TWELVE_MINIMAL_ARCHIVE_HEX),
        (13_u16, RETAINED_RECORD_THIRTEEN_MINIMAL_ARCHIVE_HEX),
        (14_u16, RETAINED_RECORD_FOURTEEN_MINIMAL_ARCHIVE_HEX),
        (16_u16, RETAINED_RECORD_SIXTEEN_MINIMAL_ARCHIVE_HEX),
        (17_u16, RETAINED_RECORD_SEVENTEEN_MINIMAL_ARCHIVE_HEX),
    ] {
        let original = decode_hex(vector.trim());
        let denial =
            decode_package_archive(&original, WorthQueryPackageArchiveLimits::DEFAULT).unwrap_err();
        assert_eq!(
            denial.kind(),
            WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion
        );
        assert_eq!(denial.compatibility().unwrap().observed_version(), version);
    }

    // package/identity.rs:81 declares canonical identity v4: hash the fixture's
    // canonical tokens with that domain, then encode the v2 archive framing.
    let export = fixture::minimal_package().export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let first = encode_package_archive(&export, limits).unwrap();
    assert_eq!(encode_package_archive(&export, limits).unwrap(), first);
    assert_eq!(encode_hex(&first), VERSION_TWO_MINIMAL_ARCHIVE_HEX.trim());
    let frozen = decode_hex(VERSION_TWO_MINIMAL_ARCHIVE_HEX.trim());
    assert_eq!(u16_at(&frozen, 8), 2, "outer archive version stays2");
    assert_eq!(u16_at(&frozen, MANIFEST_FRAME_BYTES), 18);
    let decoded = decode_package_archive(&frozen, limits).unwrap();
    assert_eq!(decoded.manifest(), export.manifest());
    assert_eq!(decoded.frames().len(), 1);
    assert_eq!(decoded.frames()[0].record(), &export.records()[0]);
}

#[test]
fn physical_and_cumulative_nested_budgets_are_symmetric() {
    let export = fixture::all_family_package()
        .export_typed_records()
        .unwrap();
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_package_archive(&export, defaults).unwrap();
    let work = decode_package_archive(&bytes, defaults)
        .unwrap()
        .decode_work();

    let exact_bytes = defaults.with_maximum_archive_bytes(bytes.len() as u64);
    assert_eq!(encode_package_archive(&export, exact_bytes).unwrap(), bytes);
    assert!(decode_package_archive(&bytes, exact_bytes).is_ok());
    let narrow_bytes = defaults.with_maximum_archive_bytes(bytes.len() as u64 - 1);
    assert_kind(
        encode_package_archive(&export, narrow_bytes).unwrap_err(),
        WorthQueryPackageArchiveDenialKind::ArchiveByteBudgetExceeded,
    );
    assert_kind(
        decode_package_archive(&bytes, narrow_bytes).unwrap_err(),
        WorthQueryPackageArchiveDenialKind::ArchiveByteBudgetExceeded,
    );

    let exact_nested = defaults.with_maximum_nested_entries(work.nested_entries());
    assert_eq!(
        encode_package_archive(&export, exact_nested).unwrap(),
        bytes
    );
    assert!(decode_package_archive(&bytes, exact_nested).is_ok());
    let narrow_nested = defaults.with_maximum_nested_entries(work.nested_entries() - 1);
    assert_kind(
        encode_package_archive(&export, narrow_nested).unwrap_err(),
        WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded,
    );
    assert_kind(
        decode_package_archive(&bytes, narrow_nested).unwrap_err(),
        WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded,
    );
}

#[test]
fn aggregate_inventory_rejects_reorder_duplicate_and_wrong_family() {
    let export = fixture::all_family_package()
        .export_typed_records()
        .unwrap();
    let bytes = encode_package_archive(&export, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let ranges = frame_ranges(&bytes);

    let mut reordered = bytes[..MANIFEST_FRAME_BYTES].to_vec();
    reordered.extend_from_slice(&bytes[ranges[1].clone()]);
    reordered.extend_from_slice(&bytes[ranges[0].clone()]);
    reordered.extend_from_slice(&bytes[ranges[2].start..]);
    assert_decode_kind(
        &reordered,
        WorthQueryPackageArchiveDenialKind::NonCanonicalRecordSequence,
    );

    let mut duplicate_index = bytes.clone();
    duplicate_index[ranges[1].start + 4..ranges[1].start + 8].copy_from_slice(&0_u32.to_be_bytes());
    assert_decode_kind(
        &duplicate_index,
        WorthQueryPackageArchiveDenialKind::NonCanonicalRecordSequence,
    );

    let capability = ranges
        .iter()
        .find(|range| u16_at(&bytes, range.start + 2) == 2)
        .unwrap();
    let mut wrong_family = bytes;
    wrong_family[capability.start + 2..capability.start + 4].copy_from_slice(&3_u16.to_be_bytes());
    assert_decode_kind(
        &wrong_family,
        WorthQueryPackageArchiveDenialKind::RecordFamilyInventoryMismatch,
    );
}

#[test]
fn aggregate_versions_truncation_trailing_and_semantic_tamper_fail_closed() {
    let source = fixture::minimal_package();
    let export = source.export_typed_records().unwrap();
    let bytes = encode_package_archive(&export, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();

    assert!(decode_package_archive(
        &bytes[..bytes.len() - 1],
        WorthQueryPackageArchiveLimits::DEFAULT
    )
    .is_err());
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_decode_kind(&trailing, WorthQueryPackageArchiveDenialKind::TrailingBytes);
    let mut archive_version = bytes.clone();
    archive_version[8..10]
        .copy_from_slice(&(WORTH_QUERY_PACKAGE_ARCHIVE_PROTOCOL_VERSION + 1).to_be_bytes());
    assert_decode_kind(
        &archive_version,
        WorthQueryPackageArchiveDenialKind::UnsupportedArchiveVersion,
    );
    let mut record_version = bytes.clone();
    record_version[MANIFEST_FRAME_BYTES..MANIFEST_FRAME_BYTES + 2]
        .copy_from_slice(&(WORTH_QUERY_PACKAGE_ARCHIVE_RECORD_PROTOCOL_VERSION - 1).to_be_bytes());
    assert_decode_kind(
        &record_version,
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion,
    );

    let mut tampered = bytes;
    let owner = b"archive.stream.minimal";
    let offset = tampered
        .windows(owner.len())
        .position(|window| window == owner)
        .unwrap();
    tampered[offset] = b'X';
    let decoded =
        decode_package_archive(&tampered, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let (manifest, frames) = decoded.into_parts();
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    for frame in frames {
        let (index, record) = frame.into_parts();
        reconstruction = reconstruction.push_record(index, record).unwrap();
    }
    assert!(reconstruction
        .close()
        .unwrap()
        .materialize()
        .unwrap()
        .validate_freshly(
            WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                source.identity().clone(),
            ),
        )
        .is_err());
}
