use crate::package::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage, WorthQueryPortableDomainPackageIdentity,
    WorthQueryPortablePackageManifest, WorthQueryPortablePackageRecord,
    WorthQueryPortablePackageRecordFamily as Family,
};
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;

use super::super::test_support::close_records;
use super::super::{
    WorthQueryPortablePackageReconstructionDenial as Denial,
    WorthQueryPortablePackageReconstructionLimits, WorthQueryReconstructedPortablePackageCandidate,
};

#[test]
fn complete_typed_package_round_trips_through_fresh_query_validation() {
    let source = crate::application_schema_tests::complete_typed_package_fixture();
    let exported = source.export_typed_records().unwrap();
    let reconstructed = semantic_candidate(exported.manifest(), exported.records().to_vec())
        .validate_freshly(expected_identity(source.identity()))
        .unwrap();

    assert!(reconstructed.has_same_authoritative_meaning(&source));
    assert_eq!(reconstructed.identity(), source.identity());
    assert_eq!(reconstructed.export_typed_records().unwrap(), exported);
}

#[test]
fn forged_manifest_identity_cannot_receive_fresh_package_authority() {
    let source = crate::application_schema_tests::complete_typed_package_fixture();
    let exported = source.export_typed_records().unwrap();
    let forged_identity =
        WorthQueryPortableDomainPackageIdentity::from_untrusted_bytes(ARCHIVE_DECODE, [0xA5; 32]);
    let forged_manifest = manifest_with(
        exported.manifest(),
        forged_identity.clone(),
        exported.manifest().logical_export_bytes(),
    );

    assert!(matches!(
        semantic_candidate(&forged_manifest, exported.records().to_vec())
            .validate_freshly(expected_identity(source.identity())),
        Err(Denial::ManifestPackageIdentityMismatch { claimed, .. }) if claimed == forged_identity
    ));
}

#[test]
fn cross_spliced_root_meaning_recomputes_a_different_package_identity() {
    let source = capability_package("source-capability");
    let foreign = capability_package("foreign-capability");
    let source_export = source.export_typed_records().unwrap();
    let foreign_export = foreign.export_typed_records().unwrap();
    let foreign_capability = foreign_export
        .records()
        .iter()
        .find(|record| record.family() == Family::CapabilityRequirement)
        .unwrap()
        .clone();
    let mut spliced_records = source_export.records().to_vec();
    let source_capability = spliced_records
        .iter_mut()
        .find(|record| record.family() == Family::CapabilityRequirement)
        .unwrap();
    *source_capability = foreign_capability;

    assert!(matches!(
        semantic_candidate(source_export.manifest(), spliced_records)
            .validate_freshly(expected_identity(source.identity())),
        Err(Denial::ManifestPackageIdentityMismatch { claimed, recomputed })
            if claimed == source.identity().clone() && recomputed == foreign.identity().clone()
    ));
}

#[test]
fn ordinary_query_validation_rejects_cross_spliced_domain_and_schema_ownership() {
    let source = crate::application_schema_tests::complete_typed_package_fixture();
    let exported = source.export_typed_records().unwrap();
    let mut records = exported.records().to_vec();
    records[0] = WorthQueryPortablePackageRecord::DomainIdentity(
        WorthQueryPortableDomainIdentity::new("foreign-owner", 1, 0),
    );

    assert!(matches!(
        semantic_candidate(exported.manifest(), records)
            .validate_freshly(expected_identity(source.identity())),
        Err(Denial::FreshPackageValidationDenied { .. })
    ));
}

#[test]
fn illegal_within_family_order_is_rejected_before_validation_can_sort_it() {
    let source = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "ordering", 1, 0,
    ))
    .requires_capability("alpha")
    .requires_capability("beta")
    .validate()
    .unwrap();
    let exported = source.export_typed_records().unwrap();
    let mut records = exported.records().to_vec();
    let capability_indices = records
        .iter()
        .enumerate()
        .filter_map(|(index, record)| {
            (record.family() == Family::CapabilityRequirement).then_some(index)
        })
        .collect::<Vec<_>>();
    records.swap(capability_indices[0], capability_indices[1]);

    assert!(matches!(
        semantic_candidate(exported.manifest(), records)
            .validate_freshly(expected_identity(source.identity())),
        Err(Denial::IllegalRecordOrdering {
            family: Family::CapabilityRequirement,
        })
    ));
}

#[test]
fn forged_native_aspect_spine_record_is_rejected_after_fresh_export() {
    let source = crate::application_schema_tests::complete_typed_package_fixture();
    let exported = source.export_typed_records().unwrap();
    let mut records = exported.records().to_vec();
    let native = records
        .iter_mut()
        .find_map(|record| match record {
            WorthQueryPortablePackageRecord::NativeAspectContract(value) => Some(value),
            _ => None,
        })
        .unwrap();
    native.replace_schema_for_test("forged-schema");

    assert!(matches!(
        semantic_candidate(exported.manifest(), records)
            .validate_freshly(expected_identity(source.identity())),
        Err(Denial::DerivedContractClosureMismatch {
            family: Family::NativeAspectContract,
        })
    ));
}

#[test]
fn forged_application_operation_spine_record_is_rejected_after_fresh_export() {
    let source = crate::application_schema_tests::complete_typed_package_fixture();
    let exported = source.export_typed_records().unwrap();
    let mut records = exported.records().to_vec();
    let operation = records
        .iter_mut()
        .find_map(|record| match record {
            WorthQueryPortablePackageRecord::ApplicationOperationContract(value) => Some(value),
            _ => None,
        })
        .unwrap();
    operation.replace_operation_for_test("forged-operation");

    assert!(matches!(
        semantic_candidate(exported.manifest(), records)
            .validate_freshly(expected_identity(source.identity())),
        Err(Denial::DerivedContractClosureMismatch {
            family: Family::ApplicationOperationContract,
        })
    ));
}

#[test]
fn forged_manifest_work_claim_is_rejected_after_fresh_export() {
    let source = crate::application_schema_tests::complete_typed_package_fixture();
    let exported = source.export_typed_records().unwrap();
    let forged_manifest = manifest_with(
        exported.manifest(),
        exported.manifest().package_identity().clone(),
        exported.manifest().logical_export_bytes() + 1,
    );

    assert!(matches!(
        semantic_candidate(&forged_manifest, exported.records().to_vec())
            .validate_freshly(expected_identity(source.identity())),
        Err(Denial::FreshManifestClosureMismatch)
    ));
}

#[test]
fn caller_expected_identity_is_distinct_from_the_manifest_claim() {
    let source = crate::application_schema_tests::complete_typed_package_fixture();
    let exported = source.export_typed_records().unwrap();
    let foreign =
        WorthQueryPortableDomainPackageIdentity::from_untrusted_bytes(ARCHIVE_DECODE, [0x5A; 32]);

    assert!(matches!(
        semantic_candidate(exported.manifest(), exported.records().to_vec())
            .validate_freshly(expected_identity(&foreign)),
        Err(Denial::ExpectedPackageIdentityMismatch { expected, recomputed })
            if expected == foreign && recomputed == source.identity().clone()
    ));
}

fn semantic_candidate(
    manifest: &WorthQueryPortablePackageManifest,
    records: Vec<WorthQueryPortablePackageRecord>,
) -> WorthQueryReconstructedPortablePackageCandidate {
    close_records(
        manifest,
        records,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .materialize()
    .unwrap()
}

fn expected_identity(
    identity: &WorthQueryPortableDomainPackageIdentity,
) -> WorthQueryExpectedPortablePackageIdentity {
    WorthQueryExpectedPortablePackageIdentity::new(identity.clone())
}

fn manifest_with(
    source: &WorthQueryPortablePackageManifest,
    identity: WorthQueryPortableDomainPackageIdentity,
    logical_export_bytes: u64,
) -> WorthQueryPortablePackageManifest {
    WorthQueryPortablePackageManifest::from_untrusted_fields(
        ARCHIVE_DECODE,
        source.version(),
        identity,
        source.record_count(),
        source.canonical_source_bytes(),
        logical_export_bytes,
        *source.family_counts(),
    )
}

fn capability_package(
    capability: &str,
) -> crate::package::WorthQueryValidatedPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "cross-splice",
        1,
        0,
    ))
    .requires_capability(capability)
    .validate()
    .unwrap()
}
