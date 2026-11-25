use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaDeclarationDenial, WorthQueryPortableApplicationSchemaRecord,
};

use crate::domain_computation::{
    WorthQueryPortableArtifactContractReadmissionDenial, WorthQueryPortableArtifactContractRecord,
};
use crate::domain_operation::WorthQueryOperationCapabilityRequirement;
use crate::package::{
    WorthQueryPortableDomainOperationParts, WorthQueryPortableDomainOperationRecord,
    WorthQueryPortableDomainOperationSemanticRecord,
    WorthQueryPortablePackageReconstructionDenial as Denial,
    WorthQueryPortablePackageReconstructionLimits, WorthQueryPortablePackageRecord,
};

use super::test_support::close_records;

#[test]
fn duplicate_nested_domain_operation_meaning_is_not_normalized_into_validity() {
    let source = operation_with_required_capability();
    let exported = source.export_typed_records().unwrap();
    let mut records = exported.records().to_vec();
    let operation = records
        .iter_mut()
        .find_map(|record| match record {
            WorthQueryPortablePackageRecord::DomainOperation(value) => Some(value),
            _ => None,
        })
        .unwrap();
    let operation_parts = operation.clone().into_parts();
    let mut semantic_parts = operation_parts.semantics.into_parts();
    let duplicate = *semantic_parts.required_capabilities.first().unwrap();
    semantic_parts.required_capabilities.push(duplicate);
    *operation = WorthQueryPortableDomainOperationRecord::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableDomainOperationParts {
            identity: operation_parts.identity,
            semantics: WorthQueryPortableDomainOperationSemanticRecord::from_untrusted_parts(
                ARCHIVE_DECODE,
                semantic_parts,
            ),
            canonical_identity: operation_parts.canonical_identity,
        },
    );

    let structural = close_records(
        exported.manifest(),
        records,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    );
    let outcome = structural.materialize();
    assert!(
        matches!(
            outcome,
            Err(Denial::NonCanonicalDomainOperationSemantics { .. })
        ),
        "{outcome:?}"
    );
}

fn operation_with_required_capability() -> crate::package::WorthQueryValidatedPortableDomainPackage
{
    let source = crate::conditional_application_operation_test_fixture::definition::<(), (), ()>();
    let identity = source.identity().clone();
    let mut semantics = source.semantics().clone();
    semantics
        .required_capabilities
        .push(WorthQueryOperationCapabilityRequirement::QueryRead);
    let operation =
        crate::domain_operation::WorthQueryDomainOperationDefinition::<(), (), ()>::new(
            identity, semantics,
        )
        .into_portable();
    crate::package::WorthQueryPortableDomainPackage::new(
        crate::package::WorthQueryPortableDomainIdentity::new("exact-readmission", 1, 0),
    )
    .domain_operation(operation)
    .validate()
    .unwrap()
}

#[test]
fn reordered_application_schema_members_are_not_normalized_into_validity() {
    let source = crate::application_schema_tests::complete_typed_package_fixture();
    let exported = source.export_typed_records().unwrap();
    let mut records = exported.records().to_vec();
    let schema = records
        .iter_mut()
        .find_map(|record| match record {
            WorthQueryPortablePackageRecord::ApplicationSchema(value) => Some(value),
            _ => None,
        })
        .unwrap();
    let mut parts = schema.clone().into_parts();
    parts.members.swap(0, 1);
    *schema =
        WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(ARCHIVE_DECODE, parts);

    let structural = close_records(
        exported.manifest(),
        records,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    );
    assert!(matches!(
        structural.materialize(),
        Err(Denial::ApplicationSchemaReadmissionDenied {
            denial: ApplicationSchemaDeclarationDenial::InvalidCanonicalOrdering,
        })
    ));
}

#[test]
fn duplicate_artifact_roles_are_not_normalized_into_validity() {
    let source = crate::application_schema_tests::complete_typed_package_fixture();
    let exported = source.export_typed_records().unwrap();
    let mut records = exported.records().to_vec();
    let contract = records
        .iter_mut()
        .find_map(|record| match record {
            WorthQueryPortablePackageRecord::ArtifactContract(value) => Some(value),
            _ => None,
        })
        .unwrap();
    let mut parts = contract.clone().into_parts();
    let duplicate = parts.producer_roles.first().unwrap().clone();
    parts.producer_roles.push(duplicate);
    *contract =
        WorthQueryPortableArtifactContractRecord::from_untrusted_parts(ARCHIVE_DECODE, parts);

    let structural = close_records(
        exported.manifest(),
        records,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    );
    assert!(matches!(
        structural.materialize(),
        Err(Denial::ArtifactContractReadmissionDenied {
            denial: WorthQueryPortableArtifactContractReadmissionDenial::NonCanonical,
        })
    ));
}

#[test]
fn oversized_schema_text_is_denied_before_canonical_basis_allocation() {
    let source = crate::application_schema_tests::complete_typed_package_fixture();
    let exported = source.export_typed_records().unwrap();
    let mut records = exported.records().to_vec();
    let schema = records
        .iter_mut()
        .find_map(|record| match record {
            WorthQueryPortablePackageRecord::ApplicationSchema(value) => Some(value),
            _ => None,
        })
        .unwrap();
    let mut parts = schema.clone().into_parts();
    parts.owner = "x".repeat(128 * 1_024);
    *schema =
        WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(ARCHIVE_DECODE, parts);
    let limits = WorthQueryPortablePackageReconstructionLimits::DEFAULT.with_work_bounds(
        1_024 * 1_024,
        u64::MAX,
        64 * 1_024,
    );

    let structural = close_records(exported.manifest(), records, limits);
    assert!(matches!(
        structural.materialize(),
        Err(Denial::ApplicationSchemaReadmissionDenied {
            denial:
                ApplicationSchemaDeclarationDenial::PortableCanonicalSourceBytesBudgetExceeded { .. },
        })
    ));
}

#[test]
fn oversized_artifact_text_is_denied_before_canonicalization_clone() {
    let source = crate::application_schema_tests::complete_typed_package_fixture();
    let exported = source.export_typed_records().unwrap();
    let mut records = exported.records().to_vec();
    let contract = records
        .iter_mut()
        .find_map(|record| match record {
            WorthQueryPortablePackageRecord::ArtifactContract(value) => Some(value),
            _ => None,
        })
        .unwrap();
    let mut parts = contract.clone().into_parts();
    parts.producer_roles = vec!["x".repeat(128 * 1_024)];
    *contract =
        WorthQueryPortableArtifactContractRecord::from_untrusted_parts(ARCHIVE_DECODE, parts);
    let limits = WorthQueryPortablePackageReconstructionLimits::DEFAULT.with_work_bounds(
        1_024 * 1_024,
        u64::MAX,
        64 * 1_024,
    );

    let structural = close_records(exported.manifest(), records, limits);
    assert!(matches!(
        structural.materialize(),
        Err(Denial::ArtifactContractReadmissionDenied {
            denial:
                WorthQueryPortableArtifactContractReadmissionDenial::CanonicalWorkBudgetExceeded { .. },
        })
    ));
}
