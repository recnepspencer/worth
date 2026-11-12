use super::test_support::{close_records, operation_fixture};
use super::*;
use crate::package::{
    WorthQueryPortableDefinition, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage, WorthQueryPortablePackageManifest,
    WorthQueryPortablePackageManifestVersion, WorthQueryPortablePackageRecordFamily,
};

#[test]
fn canonical_typed_export_closes_as_an_unvalidated_candidate() {
    let exported = fixture().export_typed_records().unwrap();
    let expected_manifest = exported.manifest().clone();
    let expected_records = exported.records().to_vec();

    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        expected_manifest.clone(),
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    for (index, record) in expected_records.iter().cloned().enumerate() {
        reconstruction = reconstruction
            .push_record(u32::try_from(index).unwrap(), record)
            .unwrap();
    }
    let candidate = reconstruction.close().unwrap();

    assert_eq!(candidate.manifest(), &expected_manifest);
    assert_eq!(candidate.records(), expected_records);
    assert_eq!(candidate.views().len(), expected_records.len());
    assert!(candidate.work().logical_bytes() > 0);
}

#[test]
fn materialization_freshly_readmits_nested_query_meaning_without_package_authority() {
    let exported = operation_fixture().export_typed_records().unwrap();
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        exported.manifest().clone(),
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    for (index, record) in exported.records().iter().cloned().enumerate() {
        reconstruction = reconstruction
            .push_record(u32::try_from(index).unwrap(), record)
            .unwrap();
    }

    let semantic = reconstruction.close().unwrap().materialize().unwrap();
    assert_eq!(semantic.domain_identity().owner(), "portable-operation");
    assert_eq!(semantic.package.domain_operations.len(), 1);
    let query = &semantic.package.domain_operations[0]
        .semantics()
        .canonical_query;
    assert_eq!(query.query().authority().digest(), query.query().digest());
    assert!(semantic.expected_native_aspects().is_empty());
    assert!(semantic.expected_application_operations().is_empty());
}

#[test]
fn materialization_rejects_forged_operation_identity_and_carried_query_budget_exhaustion() {
    let exported = operation_fixture().export_typed_records().unwrap();
    let mut forged_records = exported.records().to_vec();
    let operation = forged_records
        .iter_mut()
        .find_map(|record| match record {
            crate::package::WorthQueryPortablePackageRecord::DomainOperation(operation) => {
                Some(operation)
            }
            _ => None,
        })
        .unwrap();
    operation.replace_canonical_identity_for_test("forged-operation-identity");
    let forged = close_records(
        exported.manifest(),
        forged_records,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    );
    assert!(matches!(
        forged.materialize(),
        Err(WorthQueryPortablePackageReconstructionDenial::DomainOperationIdentityMismatch { .. })
    ));

    let narrow = WorthQueryPortablePackageReconstructionLimits::DEFAULT
        .with_canonical_query_limits(
            worth_query_declaration::facade::canonicalization::WorthQueryPortableCanonicalQueryReadmissionLimits::new(1, u64::MAX),
        );
    let limited = close_records(exported.manifest(), exported.records().to_vec(), narrow);
    assert!(matches!(
        limited.materialize(),
        Err(WorthQueryPortablePackageReconstructionDenial::CanonicalQueryReadmissionDenied {
            denial: worth_query_declaration::facade::canonicalization::QueryCanonicalizationError::PortableRecordEntryBudgetExceeded { maximum: 1, .. },
            ..
        })
    ));
}

#[test]
fn materialization_transfers_every_root_family_and_retains_populated_derived_spine() {
    let source = crate::application_schema_tests::complete_typed_package_fixture();
    let exported = source.export_typed_records().unwrap();
    let expected_native = exported
        .records()
        .iter()
        .filter_map(|record| match record {
            crate::package::WorthQueryPortablePackageRecord::NativeAspectContract(value) => {
                Some(value.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let expected_operations = exported
        .records()
        .iter()
        .filter_map(|record| match record {
            crate::package::WorthQueryPortablePackageRecord::ApplicationOperationContract(
                value,
            ) => Some(value.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(!expected_native.is_empty());
    assert!(!expected_operations.is_empty());

    let semantic = close_records(
        exported.manifest(),
        exported.records().to_vec(),
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .materialize()
    .unwrap();
    assert_eq!(&semantic.package.identity, source.domain_identity());
    assert_eq!(semantic.package.capabilities, source.capabilities());
    assert_eq!(semantic.package.configuration, source.configuration());
    assert_eq!(semantic.package.operating, source.operating_requirements());
    assert_eq!(semantic.package.definitions, source.definitions());
    assert_eq!(
        semantic.package.domain_operations,
        source.domain_operations()
    );
    assert_eq!(
        semantic.package.artifact_contracts,
        source.artifact_contracts()
    );
    assert_eq!(
        semantic.package.application_schemas,
        source.application_schemas()
    );
    assert_eq!(
        semantic.package.conditional_application_operations,
        source.conditional_application_operations()
    );
    assert_eq!(semantic.package.contributions, source.contribution_policy());
    assert_eq!(semantic.expected_native_aspects(), expected_native);
    assert_eq!(
        semantic.expected_application_operations(),
        expected_operations
    );
}

#[test]
fn materialization_rejects_a_closed_manifest_without_one_domain_root() {
    let exported = fixture().export_typed_records().unwrap();
    let mut counts = family_counts(exported.manifest());
    counts[WorthQueryPortablePackageRecordFamily::DomainIdentity as usize] = 0;
    let manifest = manifest_from(
        exported.manifest(),
        exported.manifest().version(),
        exported.manifest().record_count() - 1,
        exported.manifest().logical_export_bytes(),
        counts,
    );
    let candidate = close_records(
        &manifest,
        exported.records()[1..].to_vec(),
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    );
    assert!(matches!(
        candidate.materialize(),
        Err(
            WorthQueryPortablePackageReconstructionDenial::DomainIdentityCardinality {
                observed: 0,
            }
        )
    ));
}

#[test]
fn manifest_is_rejected_before_allocation_when_version_or_budget_is_invalid() {
    let exported = fixture().export_typed_records().unwrap();
    let unsupported = manifest_from(
        exported.manifest(),
        WorthQueryPortablePackageManifestVersion::new(1),
        exported.manifest().record_count(),
        exported.manifest().logical_export_bytes(),
        family_counts(exported.manifest()),
    );
    assert!(matches!(
        WorthQueryPortablePackageReconstruction::begin(
            unsupported,
            WorthQueryPortablePackageReconstructionLimits::DEFAULT,
        ),
        Err(WorthQueryPortablePackageReconstructionDenial::UnsupportedManifestVersion { .. })
    ));

    let record_limited =
        WorthQueryPortablePackageReconstructionLimits::new(exported.manifest().record_count() - 1);
    assert!(matches!(
        WorthQueryPortablePackageReconstruction::begin(exported.manifest().clone(), record_limited,),
        Err(WorthQueryPortablePackageReconstructionDenial::RecordBudgetExceeded { .. })
    ));

    let oversized_claim = manifest_from(
        exported.manifest(),
        exported.manifest().version(),
        exported.manifest().record_count(),
        crate::package::WORTH_QUERY_PORTABLE_PACKAGE_MAX_LOGICAL_EXPORT_BYTES + 1,
        family_counts(exported.manifest()),
    );
    assert!(matches!(
        WorthQueryPortablePackageReconstruction::begin(
            oversized_claim,
            WorthQueryPortablePackageReconstructionLimits::DEFAULT,
        ),
        Err(
            WorthQueryPortablePackageReconstructionDenial::DeclaredLogicalByteCeilingExceeded { .. }
        )
    ));
}

#[test]
fn manifest_family_total_must_equal_the_declared_record_count() {
    let exported = fixture().export_typed_records().unwrap();
    let mut counts = family_counts(exported.manifest());
    counts[WorthQueryPortablePackageRecordFamily::Definition as usize] -= 1;
    let malformed = manifest_from(
        exported.manifest(),
        exported.manifest().version(),
        exported.manifest().record_count(),
        exported.manifest().logical_export_bytes(),
        counts,
    );

    assert!(matches!(
        WorthQueryPortablePackageReconstruction::begin(
            malformed,
            WorthQueryPortablePackageReconstructionLimits::DEFAULT,
        ),
        Err(WorthQueryPortablePackageReconstructionDenial::FamilyCountMismatch { .. })
    ));
}

#[test]
fn manifest_rejects_impossible_byte_width_and_overflowing_family_total() {
    let exported = fixture().export_typed_records().unwrap();
    let impossible_width = WorthQueryPortablePackageManifest::from_untrusted_fields(
        exported.manifest().version(),
        exported.manifest().package_identity().clone(),
        exported.manifest().record_count(),
        exported.manifest().logical_export_bytes() + 1,
        exported.manifest().logical_export_bytes(),
        family_counts(exported.manifest()),
    );
    assert!(matches!(
        WorthQueryPortablePackageReconstruction::begin(
            impossible_width,
            WorthQueryPortablePackageReconstructionLimits::DEFAULT,
        ),
        Err(
            WorthQueryPortablePackageReconstructionDenial::CanonicalSourceExceedsLogicalExport { .. }
        )
    ));

    let mut overflowing_counts = [0_u32; WorthQueryPortablePackageRecordFamily::ALL.len()];
    overflowing_counts[0] = u32::MAX;
    overflowing_counts[1] = 1;
    let overflowing_families = WorthQueryPortablePackageManifest::from_untrusted_fields(
        exported.manifest().version(),
        exported.manifest().package_identity().clone(),
        0,
        0,
        0,
        overflowing_counts,
    );
    assert!(matches!(
        WorthQueryPortablePackageReconstruction::begin(
            overflowing_families,
            WorthQueryPortablePackageReconstructionLimits::DEFAULT,
        ),
        Err(WorthQueryPortablePackageReconstructionDenial::FamilyCountOverflow)
    ));
}

#[test]
fn intake_rejects_wrong_index_family_extra_record_and_incomplete_close() {
    let exported = fixture().export_typed_records().unwrap();
    let first = exported.records()[0].clone();
    let second = exported.records()[1].clone();

    let reconstruction = begin(exported.manifest());
    assert!(matches!(
        reconstruction.push_record(1, first.clone()),
        Err(WorthQueryPortablePackageReconstructionDenial::RecordIndexMismatch { .. })
    ));

    let reconstruction = begin(exported.manifest());
    assert!(matches!(
        reconstruction.push_record(0, second),
        Err(WorthQueryPortablePackageReconstructionDenial::RecordFamilyMismatch { .. })
    ));

    let reconstruction = begin(exported.manifest()).push_record(0, first).unwrap();
    assert!(matches!(
        reconstruction.close(),
        Err(WorthQueryPortablePackageReconstructionDenial::RecordCountIncomplete { .. })
    ));

    let mut reconstruction = begin(exported.manifest());
    for (index, record) in exported.records().iter().cloned().enumerate() {
        reconstruction = reconstruction
            .push_record(u32::try_from(index).unwrap(), record)
            .unwrap();
    }
    assert!(matches!(
        reconstruction.push_record(
            exported.manifest().record_count(),
            exported.records()[0].clone()
        ),
        Err(WorthQueryPortablePackageReconstructionDenial::RecordCountExceeded { .. })
    ));
}

fn begin(manifest: &WorthQueryPortablePackageManifest) -> WorthQueryPortablePackageReconstruction {
    WorthQueryPortablePackageReconstruction::begin(
        manifest.clone(),
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap()
}

fn manifest_from(
    source: &WorthQueryPortablePackageManifest,
    version: WorthQueryPortablePackageManifestVersion,
    record_count: u32,
    logical_export_bytes: u64,
    family_counts: [u32; WorthQueryPortablePackageRecordFamily::ALL.len()],
) -> WorthQueryPortablePackageManifest {
    WorthQueryPortablePackageManifest::from_untrusted_fields(
        version,
        source.package_identity().clone(),
        record_count,
        source.canonical_source_bytes(),
        logical_export_bytes,
        family_counts,
    )
}

fn family_counts(
    manifest: &WorthQueryPortablePackageManifest,
) -> [u32; WorthQueryPortablePackageRecordFamily::ALL.len()] {
    std::array::from_fn(|index| {
        manifest.family_count(WorthQueryPortablePackageRecordFamily::ALL[index])
    })
}

fn fixture() -> crate::package::WorthQueryValidatedPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "portable-reconstruction",
        1,
        0,
    ))
    .requires_capability("query-read")
    .requires_configuration("query")
    .requires_operating_posture("bounded")
    .definition(WorthQueryPortableDefinition::invariant(
        "connected",
        "one-outgoing",
    ))
    .permits_contribution("query-index")
    .validate()
    .unwrap()
}
