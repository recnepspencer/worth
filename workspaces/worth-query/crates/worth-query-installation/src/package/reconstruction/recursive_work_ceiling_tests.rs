use super::test_support::close_records;
use super::*;
use crate::package::*;
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;

#[test]
fn honest_complete_export_has_tight_intake_limits_and_round_trips_within_its_claim() {
    let source = crate::application_schema_tests::complete_typed_package_fixture();
    let exported = source.export_typed_records().unwrap();
    let observed = close_records(
        exported.manifest(),
        exported.records().to_vec(),
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .work();
    assert!(observed.logical_bytes() <= exported.manifest().logical_export_bytes());
    assert!(observed.nested_entries() > 0);
    let observed_manifest =
        manifest_with_logical_bytes(exported.manifest(), observed.logical_bytes());
    let exact_intake_limits = WorthQueryPortablePackageReconstructionLimits::DEFAULT
        .with_work_bounds(
            observed.logical_bytes(),
            observed.nested_entries(),
            WorthQueryPortablePackageReconstructionLimits::DEFAULT.maximum_canonical_work_bytes(),
        );
    let tightly_closed = close_records(
        &observed_manifest,
        exported.records().to_vec(),
        exact_intake_limits,
    );
    assert_eq!(tightly_closed.work(), observed);

    let round_trip_limits = WorthQueryPortablePackageReconstructionLimits::DEFAULT
        .with_work_bounds(
            exported.manifest().logical_export_bytes(),
            observed.nested_entries(),
            WorthQueryPortablePackageReconstructionLimits::DEFAULT.maximum_canonical_work_bytes(),
        );
    let validated = close_records(
        exported.manifest(),
        exported.records().to_vec(),
        round_trip_limits,
    )
    .materialize()
    .unwrap()
    .validate_freshly(WorthQueryExpectedPortablePackageIdentity::new(
        source.identity().clone(),
    ))
    .unwrap();
    assert!(validated.has_same_authoritative_meaning(&source));

    let logical_below = observed.logical_bytes() - 1;
    let forged_logical_manifest = manifest_with_logical_bytes(exported.manifest(), logical_below);
    assert_complete_push_denial(
        forged_logical_manifest,
        exported.records(),
        WorthQueryPortablePackageReconstructionLimits::DEFAULT.with_work_bounds(
            logical_below,
            u64::MAX,
            u64::MAX,
        ),
        |denial| {
            matches!(
                denial,
                WorthQueryPortablePackageReconstructionDenial::LogicalByteBudgetExceeded {
                    maximum,
                    ..
                } if maximum == logical_below
            )
        },
    );

    let nested_below = observed.nested_entries() - 1;
    assert_complete_push_denial(
        observed_manifest,
        exported.records(),
        WorthQueryPortablePackageReconstructionLimits::DEFAULT.with_work_bounds(
            u64::MAX,
            nested_below,
            u64::MAX,
        ),
        |denial| {
            matches!(
                denial,
                WorthQueryPortablePackageReconstructionDenial::NestedEntryBudgetExceeded {
                    maximum,
                    ..
                } if maximum == nested_below
            )
        },
    );
}

fn manifest_with_logical_bytes(
    source: &WorthQueryPortablePackageManifest,
    logical_export_bytes: u64,
) -> WorthQueryPortablePackageManifest {
    WorthQueryPortablePackageManifest::from_untrusted_fields(
        ARCHIVE_DECODE,
        source.version(),
        source.package_identity().clone(),
        source.record_count(),
        0,
        logical_export_bytes,
        std::array::from_fn(|index| {
            source.family_count(WorthQueryPortablePackageRecordFamily::ALL[index])
        }),
    )
}

fn assert_complete_push_denial(
    manifest: WorthQueryPortablePackageManifest,
    records: &[WorthQueryPortablePackageRecord],
    limits: WorthQueryPortablePackageReconstructionLimits,
    expected: impl FnOnce(WorthQueryPortablePackageReconstructionDenial) -> bool,
) {
    let mut reconstruction =
        WorthQueryPortablePackageReconstruction::begin(manifest, limits).unwrap();
    for (index, record) in records.iter().cloned().enumerate() {
        match reconstruction.push_record(u32::try_from(index).unwrap(), record) {
            Ok(next) => reconstruction = next,
            Err(denial) => {
                assert!(expected(denial));
                return;
            }
        }
    }
    panic!("sub-ceiling complete reconstruction unexpectedly admitted every record");
}
