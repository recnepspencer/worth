use super::*;
use worth_query_declaration::facade::application_schema::{
    validate_portable_application_schema_freshly, ApplicationSchemaMember,
    ErasedApplicationSchemaDeclaration, WorthQueryPortableApplicationSchemaParts,
    WorthQueryPortableApplicationSchemaRecord,
};

fn schema(name: &str, count: usize) -> ErasedApplicationSchemaDeclaration {
    let record = WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
        WorthQueryPortableApplicationSchemaParts {
            owner: "worth.commitment-test".into(),
            name: name.into(),
            major: 1,
            minor: 0,
            members: (0..count)
                .map(|index| ApplicationSchemaMember::Entity {
                    entity: format!("Entity{index:08}"),
                })
                .collect(),
            contributions: Vec::new(),
        },
    );
    validate_portable_application_schema_freshly(record).unwrap()
}

fn package() -> WorthQueryPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "worth.commitment-test",
        1,
        0,
    ))
}

fn schema_work(schema: &ErasedApplicationSchemaDeclaration) -> WorthQueryCanonicalWorkEvidence {
    crate::application_schema::derive_installed_schema_identity(schema.identity())
        .unwrap()
        .1
}

#[test]
fn complete_schema_admits_without_repeating_prefixed_source() {
    // Fixed distinct declarations keep the real child below 64 MiB while the
    // former prefixed representation exceeds it. Do not extrapolate a small
    // member's encoded width: decimal locus indexes grow with schema breadth.
    let count = 165_000;
    let declaration = schema("Large", count);
    let child_work = schema_work(&declaration);
    let fixture = package().application_schema_erased(declaration.clone());
    let validated = fixture.validate().unwrap();
    let work = validated.canonical_work();
    assert_eq!(validated.application_schemas()[0].members().len(), count);
    assert_eq!(work.digest_derivations(), 2);
    assert!(work.canonical_encoded_bytes() > child_work.canonical_encoded_bytes());
    assert!(work.canonical_encoded_bytes() <= INSTALLATION_MAXIMUM_CANONICAL_BYTES);
    assert_eq!(work.sha256_input_bytes(), work.canonical_encoded_bytes());

    // The former package representation repeated every child locus with this
    // prefix. It exhausts the same unchanged allowance for this real schema.
    let budget =
        CanonicalDigestWorkBudget::for_encoded_byte_ceiling(INSTALLATION_MAXIMUM_CANONICAL_BYTES)
            .unwrap();
    let mut old = InstallationCanonicalIdentityBasis::new(
        "worth-query.portable-domain-package",
        "worth-query-portable-domain-package-v3",
        budget,
    );
    old.embedded_basis(
        "application-schema[0].meaning",
        declaration.identity().canonical_basis(),
    )
    .unwrap();
    assert!(matches!(
        old.derive(),
        Err(CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { maximum, attempted })
            if maximum == INSTALLATION_MAXIMUM_CANONICAL_BYTES && attempted > maximum
    ));
}

#[test]
fn child_and_parent_share_exact_caller_byte_allowance() {
    let child = schema("Exact", 5);
    let child_work = schema_work(&child);
    let fixture = package().application_schema_erased(child);
    let (identity, work) =
        canonical_identity_with_maximum_bytes(&fixture, INSTALLATION_MAXIMUM_CANONICAL_BYTES)
            .unwrap();
    let exact = work.canonical_encoded_bytes();
    let (bounded, bounded_work) = canonical_identity_with_maximum_bytes(&fixture, exact).unwrap();
    assert_eq!(identity, bounded);
    assert_eq!(work, bounded_work);
    assert_eq!(work.basis_preparations(), 2);
    assert_eq!(work.digest_derivations(), 2);
    assert_eq!(work.canonical_entries(), child_work.canonical_entries() + 6);
    assert!(
        work.canonical_material_allocation_bytes()
            > child_work.canonical_material_allocation_bytes()
    );
    let parent_bytes = exact - child_work.canonical_encoded_bytes();
    assert_eq!(
        work.sha256_compression_blocks(),
        child_work.sha256_compression_blocks() + (parent_bytes + 9).div_ceil(64)
    );
    assert_eq!(
        canonical_identity_with_maximum_bytes(&fixture, exact - 1).unwrap_err(),
        CanonicalDigestDerivationDenial::EncodedByteLimitExceeded {
            maximum: exact - 1,
            attempted: exact,
        }
    );
}

#[test]
fn multiple_children_share_budget_instead_of_resetting_it() {
    let first = schema("First", 20);
    let second = schema("Second", 20);
    let allowance = schema_work(&first).canonical_encoded_bytes() + 1;
    let fixture = package()
        .application_schema_erased(first)
        .application_schema_erased(second);
    assert!(matches!(
        canonical_identity_with_maximum_bytes(&fixture, allowance),
        Err(CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { maximum, attempted })
            if maximum == allowance && attempted > allowance
    ));
}

#[test]
fn admitted_child_meaning_changes_identity_and_source_order_does_not() {
    let first = schema("First", 2);
    let second = schema("Second", 2);
    let source = package()
        .application_schema_erased(first.clone())
        .application_schema_erased(second.clone())
        .validate()
        .unwrap();
    let reordered = package()
        .application_schema_erased(second)
        .application_schema_erased(first)
        .validate()
        .unwrap();
    assert_eq!(source.identity(), reordered.identity());
    assert_eq!(source.canonical_work().digest_derivations(), 3);
    let changed = package()
        .application_schema_erased(schema("First", 3))
        .application_schema_erased(schema("Second", 2))
        .validate()
        .unwrap();
    assert_ne!(source.identity(), changed.identity());
}

#[test]
fn fresh_reconstruction_rejects_cross_spliced_schema_source() {
    use crate::package::{
        WorthQueryExpectedPortablePackageIdentity, WorthQueryPortablePackageReconstruction,
        WorthQueryPortablePackageReconstructionDenial,
        WorthQueryPortablePackageReconstructionLimits, WorthQueryPortablePackageRecord,
    };

    let original = schema("Splice", 2);
    let mut parts = WorthQueryPortableApplicationSchemaRecord::project(&original).into_parts();
    parts.members[1] = ApplicationSchemaMember::Entity {
        entity: "Entity00000009".into(),
    };
    let foreign = validate_portable_application_schema_freshly(
        WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
    )
    .unwrap();
    let source = package()
        .application_schema_erased(original)
        .validate()
        .unwrap();
    let foreign = package()
        .application_schema_erased(foreign)
        .validate()
        .unwrap();
    let exported = source.export_typed_records().unwrap();
    let foreign_export = foreign.export_typed_records().unwrap();
    let changed = foreign_export
        .records()
        .iter()
        .find(|record| {
            matches!(
                record,
                WorthQueryPortablePackageRecord::ApplicationSchema(_)
            )
        })
        .unwrap();
    let mut records = exported.records().to_vec();
    let retained = records
        .iter_mut()
        .find(|record| {
            matches!(
                record,
                WorthQueryPortablePackageRecord::ApplicationSchema(_)
            )
        })
        .unwrap();
    *retained = changed.clone();
    let mut intake = WorthQueryPortablePackageReconstruction::begin(
        exported.manifest().clone(),
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    for (index, record) in records.into_iter().enumerate() {
        intake = intake
            .push_record(u32::try_from(index).unwrap(), record)
            .unwrap();
    }
    let candidate = intake.close().unwrap().materialize().unwrap();
    assert!(matches!(
        candidate.validate_freshly(
            WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                source.identity().clone(),
            ),
        ),
        Err(WorthQueryPortablePackageReconstructionDenial::ManifestPackageIdentityMismatch {
            claimed, recomputed,
        }) if claimed == source.identity().clone() && recomputed == foreign.identity().clone()
    ));
}
