use worth_foundational::facade::{
    canonicalization, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestWorkBudget, InternedString,
};

use super::{MutationSchema, RenameOutputs};
use crate::application_operation::{
    ApplicationMutationDescription, ApplicationMutationOutputContract,
    ApplicationMutationOutputRoleCardinality,
};
use crate::application_schema::{
    validate_portable_application_schema_freshly, ApplicationSchemaIdentity,
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
    WorthQueryPortableApplicationSchemaRecord,
};

#[test]
fn fixed_roles_are_exactly_one_unless_declared_optional() {
    let roles = <RenameOutputs as ApplicationMutationOutputContract<MutationSchema>>::ROLES;

    assert_eq!(roles[0].name(), "renamed-account");
    assert_eq!(
        roles[0].cardinality(),
        ApplicationMutationOutputRoleCardinality::ExactlyOne
    );
    assert!(!roles[0].cardinality().admits_absence());
    assert_eq!(roles[1].name(), "retired-mapping");
    assert_eq!(roles[1].entity(), "ExternalMapping");
    assert_eq!(
        roles[1].cardinality(),
        ApplicationMutationOutputRoleCardinality::AtMostOne
    );
    assert!(roles[1].cardinality().admits_absence());
}

#[test]
fn portable_readmission_carries_each_role_cardinality() {
    let restored = readmitted(portable_record());

    assert_eq!(
        role_cardinalities(mutation_description(&restored)),
        [
            (
                "renamed-account",
                ApplicationMutationOutputRoleCardinality::ExactlyOne
            ),
            (
                "retired-mapping",
                ApplicationMutationOutputRoleCardinality::AtMostOne
            ),
        ]
    );
}

#[test]
fn role_cardinality_is_part_of_the_canonical_schema_identity() {
    let source = MutationSchema::declaration().unwrap();
    let optional = readmitted(portable_record());
    let exact = readmitted(with_every_role(
        portable_record(),
        ApplicationMutationOutputRoleCardinality::ExactlyOne,
    ));

    assert_eq!(source.identity(), optional.identity());
    assert_eq!(
        cardinality_loci(&optional),
        ["exactly-one".to_owned(), "at-most-one".to_owned()]
    );
    assert_eq!(
        cardinality_loci(&exact),
        ["exactly-one".to_owned(), "exactly-one".to_owned()]
    );
    assert_ne!(optional.identity(), exact.identity());
    assert_eq!(digest(source.identity()), digest(optional.identity()));
    assert_ne!(digest(optional.identity()), digest(exact.identity()));
}

fn digest(identity: &ApplicationSchemaIdentity) -> [u8; 32] {
    let budget = CanonicalDigestWorkBudget::new(4_096, 1024 * 1024)
        .expect("the schema digest budget is nonzero");
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(
            identity.canonical_basis().clone(),
            CanonicalDigestAlgorithmId::sha256(),
            budget,
        )
        .into_result()
        .expect("the fixture schema fits the digest budget");
    *canonicalization().digest().derive(ready).value().bytes()
}

fn portable_record() -> WorthQueryPortableApplicationSchemaRecord {
    WorthQueryPortableApplicationSchemaRecord::project(
        MutationSchema::declaration().unwrap().erased(),
    )
}

fn readmitted(
    record: WorthQueryPortableApplicationSchemaRecord,
) -> ErasedApplicationSchemaDeclaration {
    validate_portable_application_schema_freshly(record).expect("the schema readmits")
}

fn with_every_role(
    record: WorthQueryPortableApplicationSchemaRecord,
    cardinality: ApplicationMutationOutputRoleCardinality,
) -> WorthQueryPortableApplicationSchemaRecord {
    let mut parts = record.into_parts();
    for member in &mut parts.members {
        if let ApplicationSchemaMember::ApplicationMutation { description } = member {
            let mut description_parts = description.clone().into_parts();
            for role in &mut description_parts.output_roles {
                role.cardinality = cardinality;
            }
            *description = ApplicationMutationDescription::from_untrusted_parts(description_parts);
        }
    }
    WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts)
}

fn mutation_description(
    declaration: &ErasedApplicationSchemaDeclaration,
) -> &ApplicationMutationDescription {
    declaration
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::ApplicationMutation { description } => Some(description),
            _ => None,
        })
        .expect("the schema declares one mutation")
}

fn role_cardinalities(
    description: &ApplicationMutationDescription,
) -> Vec<(&str, ApplicationMutationOutputRoleCardinality)> {
    description
        .output_roles()
        .iter()
        .map(|role| (role.name.as_str(), role.cardinality))
        .collect()
}

fn cardinality_loci(declaration: &ErasedApplicationSchemaDeclaration) -> Vec<String> {
    declaration
        .identity()
        .canonical_basis()
        .payload()
        .entries()
        .iter()
        .filter_map(|entry| match (entry.locus(), entry.value()) {
            (
                CanonicalBasisLocus::Named(InternedString::Raw(locus)),
                CanonicalBasisValue::ExactText(InternedString::Raw(value)),
            ) if locus.contains(".output-role[") && locus.ends_with(".cardinality") => {
                Some(value.to_string())
            }
            _ => None,
        })
        .collect()
}
