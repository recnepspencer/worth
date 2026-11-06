//! Corrupt portable metadata exercises installation guards after SDK authoring.
use super::{
    admitted_package, assert_package_contract_denial, WorthQueryPortablePackageValidationDenialKind,
};
use worth_foundational::facade::{AspectContractRevision, AspectIdentity, ScalarAspectType};
use worth_query_declaration::facade::application_schema::*;
mod model;
pub(super) use model::*;

pub(super) fn fresh(
    edit: impl FnOnce(&mut Vec<ApplicationSchemaMember>),
) -> ErasedApplicationSchemaDeclaration {
    let declaration = SourceSchema::declaration().unwrap();
    let mut parts =
        WorthQueryPortableApplicationSchemaRecord::project(declaration.erased()).into_parts();
    edit(&mut parts.members);
    parts.members.sort();
    validate_portable_application_schema_freshly(
        WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
    )
    .unwrap()
}
#[test]
fn duplicate_semantic_aspect_locus_denies_instead_of_overwriting_meaning() {
    let declaration = fresh(|members| {
        let mut duplicate = members.iter().find(|member| matches!(member, ApplicationSchemaMember::Aspect { entity, .. } if entity == "FirstNamedEntity")).unwrap().clone();
        if let ApplicationSchemaMember::Aspect {
            identity, revision, ..
        } = &mut duplicate
        {
            identity.0 ^= 1;
            *revision = AspectContractRevision(revision.0.wrapping_add(1));
        }
        members.push(duplicate);
    });
    assert_package_contract_denial(
        declaration,
        WorthQueryPortablePackageValidationDenialKind::ApplicationContractDuplicateAspectLocus,
    );
}
#[test]
fn duplicate_semantic_field_locus_denies_instead_of_overwriting_shape() {
    let declaration = fresh(|members| {
        let mut duplicate = members.iter().find(|member| matches!(member, ApplicationSchemaMember::Field { entity, .. } if entity == "FieldEntity")).unwrap().clone();
        if let ApplicationSchemaMember::Field {
            scalar_family,
            value_type,
            ..
        } = &mut duplicate
        {
            *scalar_family = ScalarAspectType::String;
            *value_type = StringApplicationValueBinding::IDENTITY_NAME.to_owned();
        }
        members.push(duplicate);
    });
    assert_package_contract_denial(
        declaration,
        WorthQueryPortablePackageValidationDenialKind::ApplicationContractDuplicateFieldLocus,
    );
}
#[test]
fn duplicate_identity_denies_across_distinct_entities() {
    let declaration = fresh(|members| {
        let identity = members
            .iter()
            .find_map(|member| match member {
                ApplicationSchemaMember::Aspect {
                    entity, identity, ..
                } if entity == "FirstNamedEntity" => Some(*identity),
                _ => None,
            })
            .unwrap();
        for member in members {
            if let ApplicationSchemaMember::Aspect {
                entity,
                identity: other,
                ..
            } = member
                && entity == "SecondNamedEntity"
            {
                *other = identity;
            }
        }
    });
    assert_package_contract_denial(
        declaration,
        WorthQueryPortablePackageValidationDenialKind::ApplicationContractDuplicateAspectIdentity,
    );
}
#[test]
fn same_rendered_aspect_and_field_names_remain_distinct_across_entities() {
    let declaration = SourceSchema::declaration().unwrap();
    let index = super::WorthQueryInstalledPackageIndex::build(
        super::WorthQueryInstallationRuntimeIdentity::fresh(),
        super::WorthQueryInstallationGeneration::initial(),
        [admitted_package(declaration.clone())],
    )
    .unwrap();
    let installed = index.bind_application_schema(declaration).unwrap();
    let first = installed
        .native_contracts()
        .aspect("FirstNamedEntity", FirstField::at("SameField").aspect())
        .unwrap();
    let second = installed
        .native_contracts()
        .aspect("SecondNamedEntity", SecondField::at("SameField").aspect())
        .unwrap();
    assert_ne!(first.contract().identity(), second.contract().identity());
    assert_eq!(
        first.fields().cloned().collect::<Vec<_>>(),
        second.fields().cloned().collect::<Vec<_>>()
    );
    assert_eq!(
        first.contract().identity(),
        AspectIdentity(super::name_identity(
            "collisions.FirstNamedEntity.SameAspect"
        ))
    );
}
