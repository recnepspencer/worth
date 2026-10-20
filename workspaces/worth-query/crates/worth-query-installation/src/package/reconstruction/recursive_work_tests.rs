use std::collections::BTreeSet;

use worth_foundational::facade::{
    AspectBinding, AspectMask, CanonicalFieldPath, FieldKey, ProjectionMask,
};

use super::*;
use crate::package::*;

const NARROW_LOGICAL_BYTES: u64 = 64 * 1024;

#[test]
fn native_binding_bytes_are_charged_before_the_record_is_retained() {
    let exported = fixture_export();
    let (domain, native) = domain_and_family(
        exported.records(),
        WorthQueryPortablePackageRecordFamily::NativeAspectContract,
    );
    let WorthQueryPortablePackageRecord::NativeAspectContract(native) = native else {
        unreachable!()
    };
    let hostile = WorthQueryPortableNativeAspectContractRecord::from_untrusted_parts(
        WorthQueryPortableNativeAspectContractParts {
            schema: native.schema().to_owned(),
            entity: native.entity().to_owned(),
            aspect: native.aspect().clone(),
            contract: native.contract().clone(),
            fields: native.fields().cloned().collect::<BTreeSet<_>>(),
            binding: AspectBinding::EntityField {
                field: FieldKey::new("b".repeat(1024 * 1024)).unwrap(),
            },
        },
    );
    assert_logical_push_denial(
        exported.manifest(),
        domain,
        WorthQueryPortablePackageRecord::NativeAspectContract(hostile),
        WorthQueryPortablePackageRecordFamily::NativeAspectContract,
    );
}

#[test]
fn application_projection_mask_paths_are_charged_before_retention() {
    let exported = fixture_export();
    let (_, native) = domain_and_family(
        exported.records(),
        WorthQueryPortablePackageRecordFamily::NativeAspectContract,
    );
    let (domain, operation) = domain_and_family(
        exported.records(),
        WorthQueryPortablePackageRecordFamily::ApplicationOperationContract,
    );
    let WorthQueryPortablePackageRecord::NativeAspectContract(native) = native else {
        unreachable!()
    };
    let WorthQueryPortablePackageRecord::ApplicationOperationContract(operation) = operation else {
        unreachable!()
    };
    let baseline_nested_entries = operation.reconstruction_work().nested_entries;
    let paths = (0..128).map(|index| {
        CanonicalFieldPath::single(
            FieldKey::new(format!("field-{index}-{}", "p".repeat(1024))).unwrap(),
        )
    });
    let hostile = WorthQueryPortableApplicationOperationContractRecord::from_untrusted_parts(
        WorthQueryPortableApplicationOperationContractParts {
            schema: operation.schema().to_owned(),
            operation: operation.operation().to_owned(),
            input_type: operation.input_type().clone(),
            graph_reads: vec![
                WorthQueryPortableOperationGraphReadScope::NativeProjection {
                    schema: operation.schema().to_owned(),
                    entity: native.entity().to_owned(),
                    aspect: native.aspect().clone(),
                    contract: native.contract().clone(),
                    mask: AspectMask::<ProjectionMask>::new(paths),
                },
            ],
            touches: operation.touches().to_vec(),
            emissions: operation.emissions().to_vec(),
            external_effect: operation.external_effect().cloned(),
            reconciliation: operation.reconciliation().cloned(),
            body: operation.body().cloned(),
        },
    );
    assert_nested_growth_push_denial(
        exported.manifest(),
        domain.clone(),
        WorthQueryPortablePackageRecord::ApplicationOperationContract(hostile.clone()),
        WorthQueryPortablePackageRecordFamily::ApplicationOperationContract,
        baseline_nested_entries,
    );
    assert_logical_push_denial(
        exported.manifest(),
        domain,
        WorthQueryPortablePackageRecord::ApplicationOperationContract(hostile),
        WorthQueryPortablePackageRecordFamily::ApplicationOperationContract,
    );
}

#[test]
fn artifact_migration_owner_entries_are_charged_before_retention() {
    let exported = fixture_export();
    let (domain, artifact) = domain_and_family(
        exported.records(),
        WorthQueryPortablePackageRecordFamily::ArtifactContract,
    );
    let WorthQueryPortablePackageRecord::ArtifactContract(artifact) = artifact else {
        unreachable!()
    };
    let baseline_nested_entries = artifact.reconstruction_work().1;
    let mut parts = artifact.into_parts();
    for index in 0..128 {
        parts.compatibility = parts
            .compatibility
            .migration_owner(format!("migration-owner-{index}"));
    }
    assert_nested_growth_push_denial(
        exported.manifest(),
        domain,
        WorthQueryPortablePackageRecord::ArtifactContract(
            crate::domain_computation::WorthQueryPortableArtifactContractRecord::from_untrusted_parts(parts),
        ),
        WorthQueryPortablePackageRecordFamily::ArtifactContract,
        baseline_nested_entries,
    );
}

#[test]
fn operation_collection_entries_are_charged_before_retention() {
    let exported = fixture_export();
    let (domain, operation) = domain_and_family(
        exported.records(),
        WorthQueryPortablePackageRecordFamily::DomainOperation,
    );
    let WorthQueryPortablePackageRecord::DomainOperation(operation) = operation else {
        unreachable!()
    };
    let mut parts = operation.into_parts();
    let mut semantics = parts.semantics.into_parts();
    semantics.collection =
        crate::domain_operation::WorthQueryOperationCollectionContract::Collection {
            row_identity_field:
                crate::domain_operation::WorthQueryOperationCollectionField::from_dotted(
                    "account.identity",
                )
                .unwrap(),
            ordering_fields: Vec::new(),
            grouping: crate::domain_operation::WorthQueryOperationGroupingContract::Ungrouped,
            window: crate::domain_operation::WorthQueryOperationWindowPolicy::CompleteCollection,
            continuation:
                crate::domain_operation::WorthQueryOperationContinuationPosture::NotRequired,
        };
    parts.semantics =
        WorthQueryPortableDomainOperationSemanticRecord::from_untrusted_parts(semantics);
    let shell = WorthQueryPortableDomainOperationRecord::from_untrusted_parts(parts);
    let baseline_nested_entries = shell.reconstruction_work().1;
    let mut parts = shell.into_parts();
    let mut semantics = parts.semantics.into_parts();
    let crate::domain_operation::WorthQueryOperationCollectionContract::Collection {
        ordering_fields,
        ..
    } = &mut semantics.collection
    else {
        unreachable!()
    };
    ordering_fields.extend((0..128).map(|index| {
        crate::domain_operation::WorthQueryOperationCollectionField::from_dotted(&format!(
            "account.field{index}"
        ))
        .unwrap()
    }));
    parts.semantics =
        WorthQueryPortableDomainOperationSemanticRecord::from_untrusted_parts(semantics);
    assert_nested_growth_push_denial(
        exported.manifest(),
        domain,
        WorthQueryPortablePackageRecord::DomainOperation(
            WorthQueryPortableDomainOperationRecord::from_untrusted_parts(parts),
        ),
        WorthQueryPortablePackageRecordFamily::DomainOperation,
        baseline_nested_entries,
    );
}

fn fixture_export() -> WorthQueryPortablePackageRecordSet {
    crate::application_schema_tests::complete_typed_package_fixture()
        .export_typed_records()
        .unwrap()
}

fn domain_and_family(
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

fn assert_logical_push_denial(
    source: &WorthQueryPortablePackageManifest,
    domain: WorthQueryPortablePackageRecord,
    hostile: WorthQueryPortablePackageRecord,
    family: WorthQueryPortablePackageRecordFamily,
) {
    assert_push_denial(
        source,
        domain,
        hostile,
        family,
        NARROW_LOGICAL_BYTES,
        u64::MAX,
        |denial| {
            matches!(
                denial,
                WorthQueryPortablePackageReconstructionDenial::LogicalByteBudgetExceeded {
                    maximum: NARROW_LOGICAL_BYTES,
                    ..
                }
            )
        },
    );
}

fn assert_nested_growth_push_denial(
    source: &WorthQueryPortablePackageManifest,
    domain: WorthQueryPortablePackageRecord,
    hostile: WorthQueryPortablePackageRecord,
    family: WorthQueryPortablePackageRecordFamily,
    baseline_nested_entries: u64,
) {
    assert_push_denial(
        source,
        domain,
        hostile,
        family,
        u64::MAX,
        baseline_nested_entries,
        |denial| {
            matches!(
                denial,
                WorthQueryPortablePackageReconstructionDenial::NestedEntryBudgetExceeded {
                    maximum,
                    ..
                } if maximum == baseline_nested_entries
            )
        },
    );
}

fn assert_push_denial(
    source: &WorthQueryPortablePackageManifest,
    domain: WorthQueryPortablePackageRecord,
    hostile: WorthQueryPortablePackageRecord,
    family: WorthQueryPortablePackageRecordFamily,
    maximum_logical_bytes: u64,
    maximum_nested_entries: u64,
    expected: impl FnOnce(WorthQueryPortablePackageReconstructionDenial) -> bool,
) {
    let mut counts = [0; WorthQueryPortablePackageRecordFamily::ALL.len()];
    counts[WorthQueryPortablePackageRecordFamily::DomainIdentity as usize] = 1;
    counts[family as usize] = 1;
    let manifest = WorthQueryPortablePackageManifest::from_untrusted_fields(
        source.version(),
        source.package_identity().clone(),
        2,
        0,
        NARROW_LOGICAL_BYTES,
        counts,
    );
    let limits = WorthQueryPortablePackageReconstructionLimits::DEFAULT.with_work_bounds(
        maximum_logical_bytes,
        maximum_nested_entries,
        u64::MAX,
    );
    let reconstruction = WorthQueryPortablePackageReconstruction::begin(manifest, limits)
        .unwrap()
        .push_record(0, domain)
        .unwrap();
    let denial = match reconstruction.push_record(1, hostile) {
        Ok(_) => panic!("hostile recursive work unexpectedly entered reconstruction"),
        Err(denial) => denial,
    };
    assert!(expected(denial));
}
