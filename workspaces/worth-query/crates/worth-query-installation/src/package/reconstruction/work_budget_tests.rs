use super::test_support::{close_records, operation_fixture};
use super::*;
use crate::package::{
    WorthQueryPortableDefinition, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage, WorthQueryPortablePackageManifest,
    WorthQueryPortablePackageRecord, WorthQueryPortablePackageRecordFamily,
};
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;

const HOSTILE_LOGICAL_LIMIT: u64 = 64 * 1024;
const HOSTILE_TEXT_BYTES: usize = 1024 * 1024;

#[test]
fn forged_small_manifest_cannot_bypass_actual_logical_byte_admission() {
    let exported = fixture().export_typed_records().unwrap();
    let forged = WorthQueryPortablePackageManifest::from_untrusted_fields(
        ARCHIVE_DECODE,
        exported.manifest().version(),
        exported.manifest().package_identity().clone(),
        exported.manifest().record_count(),
        0,
        64,
        family_counts(exported.manifest()),
    );
    let limits = WorthQueryPortablePackageReconstructionLimits::DEFAULT.with_work_bounds(
        64,
        u64::MAX,
        u64::MAX,
    );
    let reconstruction = WorthQueryPortablePackageReconstruction::begin(forged, limits).unwrap();
    let reconstruction = reconstruction
        .push_record(0, exported.records()[0].clone())
        .unwrap();
    assert!(matches!(
        reconstruction.push_record(1, exported.records()[1].clone()),
        Err(
            WorthQueryPortablePackageReconstructionDenial::LogicalByteBudgetExceeded {
                maximum: 64,
                ..
            }
        )
    ));
}

#[test]
fn oversized_schema_owner_is_denied_at_push_before_records_can_close() {
    let exported = crate::application_schema_tests::complete_typed_package_fixture()
        .export_typed_records()
        .unwrap();
    let (domain, schema) = domain_and_family_record(
        exported.records(),
        WorthQueryPortablePackageRecordFamily::ApplicationSchema,
    );
    let WorthQueryPortablePackageRecord::ApplicationSchema(schema) = schema else {
        unreachable!()
    };
    let mut parts = schema.into_parts();
    parts.owner = "x".repeat(HOSTILE_TEXT_BYTES);
    let schema = WorthQueryPortablePackageRecord::ApplicationSchema(
        worth_query_declaration::facade::application_schema::WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(ARCHIVE_DECODE, parts),
    );
    assert_push_logical_denial(
        exported.manifest(),
        domain,
        schema,
        WorthQueryPortablePackageRecordFamily::ApplicationSchema,
    );
}

#[test]
fn oversized_artifact_evidence_is_denied_at_push_before_records_can_close() {
    let exported = crate::application_schema_tests::complete_typed_package_fixture()
        .export_typed_records()
        .unwrap();
    let (domain, artifact) = domain_and_family_record(
        exported.records(),
        WorthQueryPortablePackageRecordFamily::ArtifactContract,
    );
    let WorthQueryPortablePackageRecord::ArtifactContract(artifact) = artifact else {
        unreachable!()
    };
    let mut parts = artifact.into_parts();
    parts.evidence = crate::domain_computation::WorthQueryArtifactEvidenceContract::new(
        "e".repeat(HOSTILE_TEXT_BYTES),
        "provenance",
        "dependency",
        "invalidation",
        "equivalence",
    );
    let artifact = WorthQueryPortablePackageRecord::ArtifactContract(
        crate::domain_computation::WorthQueryPortableArtifactContractRecord::from_untrusted_parts(
            ARCHIVE_DECODE,
            parts,
        ),
    );
    assert_push_logical_denial(
        exported.manifest(),
        domain,
        artifact,
        WorthQueryPortablePackageRecordFamily::ArtifactContract,
    );
}

#[test]
fn oversized_operation_workflow_identity_is_denied_at_push_before_records_can_close() {
    let exported = operation_fixture().export_typed_records().unwrap();
    let (domain, operation) = domain_and_family_record(
        exported.records(),
        WorthQueryPortablePackageRecordFamily::DomainOperation,
    );
    let WorthQueryPortablePackageRecord::DomainOperation(operation) = operation else {
        unreachable!()
    };
    let mut parts = operation.into_parts();
    let mut semantics = parts.semantics.into_parts();
    semantics.workflow = crate::domain_operation::WorthQueryOperationWorkflowContract::Declared(
        crate::domain_operation::WorthQueryPortableWorkflowDefinition::new(
            "w".repeat(HOSTILE_TEXT_BYTES),
            Vec::<crate::domain_operation::WorthQueryPortableWorkflowStage>::new(),
        ),
    );
    parts.semantics =
        crate::package::WorthQueryPortableDomainOperationSemanticRecord::from_untrusted_parts(
            ARCHIVE_DECODE,
            semantics,
        );
    let operation = WorthQueryPortablePackageRecord::DomainOperation(
        crate::package::WorthQueryPortableDomainOperationRecord::from_untrusted_parts(
            ARCHIVE_DECODE,
            parts,
        ),
    );
    assert_push_logical_denial(
        exported.manifest(),
        domain,
        operation,
        WorthQueryPortablePackageRecordFamily::DomainOperation,
    );
}

#[test]
fn nested_query_entries_are_bounded_before_semantic_materialization() {
    let exported = operation_fixture().export_typed_records().unwrap();
    let limits = WorthQueryPortablePackageReconstructionLimits::DEFAULT.with_work_bounds(
        u64::MAX,
        1,
        u64::MAX,
    );
    let mut reconstruction =
        WorthQueryPortablePackageReconstruction::begin(exported.manifest().clone(), limits)
            .unwrap();
    let mut denial = None;
    for (index, record) in exported.records().iter().cloned().enumerate() {
        match reconstruction.push_record(u32::try_from(index).unwrap(), record) {
            Ok(next) => reconstruction = next,
            Err(observed) => {
                denial = Some(observed);
                break;
            }
        }
    }
    assert!(matches!(
        denial,
        Some(
            WorthQueryPortablePackageReconstructionDenial::NestedEntryBudgetExceeded {
                maximum: 1,
                ..
            }
        )
    ));
}

#[test]
fn forged_small_canonical_claim_cannot_widen_materialization_work() {
    let exported = operation_fixture().export_typed_records().unwrap();
    let forged = WorthQueryPortablePackageManifest::from_untrusted_fields(
        ARCHIVE_DECODE,
        exported.manifest().version(),
        exported.manifest().package_identity().clone(),
        exported.manifest().record_count(),
        0,
        exported.manifest().logical_export_bytes(),
        family_counts(exported.manifest()),
    );
    let limits = WorthQueryPortablePackageReconstructionLimits::DEFAULT.with_work_bounds(
        u64::MAX,
        u64::MAX,
        1,
    );
    let candidate = close_records(&forged, exported.records().to_vec(), limits);
    assert!(matches!(
        candidate.materialize(),
        Err(WorthQueryPortablePackageReconstructionDenial::CanonicalQueryReadmissionDenied {
            denial: worth_query_declaration::facade::canonicalization::QueryCanonicalizationError::PortableRecordLogicalBytesBudgetExceeded {
                maximum: 1,
                ..
            },
            ..
        })
    ));
}

fn family_counts(
    manifest: &WorthQueryPortablePackageManifest,
) -> [u32; WorthQueryPortablePackageRecordFamily::ALL.len()] {
    std::array::from_fn(|index| {
        manifest.family_count(WorthQueryPortablePackageRecordFamily::ALL[index])
    })
}

fn domain_and_family_record(
    records: &[WorthQueryPortablePackageRecord],
    family: WorthQueryPortablePackageRecordFamily,
) -> (
    WorthQueryPortablePackageRecord,
    WorthQueryPortablePackageRecord,
) {
    let domain = records
        .iter()
        .find(|record| record.family() == WorthQueryPortablePackageRecordFamily::DomainIdentity)
        .unwrap()
        .clone();
    let selected = records
        .iter()
        .find(|record| record.family() == family)
        .unwrap()
        .clone();
    (domain, selected)
}

fn assert_push_logical_denial(
    source: &WorthQueryPortablePackageManifest,
    domain: WorthQueryPortablePackageRecord,
    hostile: WorthQueryPortablePackageRecord,
    family: WorthQueryPortablePackageRecordFamily,
) {
    let mut counts = [0; WorthQueryPortablePackageRecordFamily::ALL.len()];
    counts[WorthQueryPortablePackageRecordFamily::DomainIdentity as usize] = 1;
    counts[family as usize] = 1;
    let manifest = WorthQueryPortablePackageManifest::from_untrusted_fields(
        ARCHIVE_DECODE,
        source.version(),
        source.package_identity().clone(),
        2,
        0,
        HOSTILE_LOGICAL_LIMIT,
        counts,
    );
    let limits = WorthQueryPortablePackageReconstructionLimits::DEFAULT.with_work_bounds(
        HOSTILE_LOGICAL_LIMIT,
        u64::MAX,
        u64::MAX,
    );
    let reconstruction = WorthQueryPortablePackageReconstruction::begin(manifest, limits)
        .unwrap()
        .push_record(0, domain)
        .unwrap();
    assert!(matches!(
        reconstruction.push_record(1, hostile),
        Err(
            WorthQueryPortablePackageReconstructionDenial::LogicalByteBudgetExceeded {
                maximum: HOSTILE_LOGICAL_LIMIT,
                ..
            }
        )
    ));
}

fn fixture() -> crate::package::WorthQueryValidatedPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "portable-reconstruction-work",
        1,
        0,
    ))
    .requires_capability("query-read")
    .definition(WorthQueryPortableDefinition::invariant(
        "connected",
        "one-outgoing",
    ))
    .validate()
    .unwrap()
}
