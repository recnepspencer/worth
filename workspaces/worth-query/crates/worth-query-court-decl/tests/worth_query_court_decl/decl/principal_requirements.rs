//! Composition owns principal field and posture refusals.
use worth_query_testkit_decl::principal_requirements_fixture::*;
fn refused<const D: u8>(entity: &str, aspect: &str, path: &str) {
    let expected = ApplicationFieldBindingLocus::new(entity, aspect, path);
    let denial = Model::<D>::declaration().unwrap_err();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::InvalidPrincipalBindingField {
            binding: "actor".into(),
            field: expected.clone()
        }
    );
    let message = denial.to_string();
    for part in [
        "actor",
        expected.entity(),
        expected.aspect(),
        expected.field(),
    ] {
        assert!(message.contains(part), "{message}");
    }
}
#[test]
fn lawful_principal_requirements_compose() {
    Model::<0>::declaration().unwrap();
}
#[test]
fn misspelled_mapping_identity_is_refused_at_composition() {
    refused::<1>("mapping", FieldSelection::<Model<0>, Mapping<0>, External<0>, WorthQueryExternalPrincipalIdentity>::at("typo").aspect(), "typo");
}
#[test]
fn foreign_mapping_identity_aspect_is_refused_at_composition() {
    refused::<2>("mapping", FieldSelection::<Model<0>, Mapping<0>, External<0>, WorthQueryExternalPrincipalIdentity>::at("value").aspect(), "value");
}
#[test]
fn writable_mapping_identity_is_refused_at_composition() {
    refused::<3>("mapping", FieldSelection::<Model<0>, Mapping<0>, External<0>, WorthQueryExternalPrincipalIdentity>::at("value").aspect(), "value");
}
#[test]
fn read_only_mapping_status_is_refused_at_composition() {
    refused::<4>(
        "mapping",
        FieldSelection::<Model<0>, Mapping<0>, Status<0>, WorthQueryPrincipalMappingStatus>::at(
            "value",
        )
        .aspect(),
        "value",
    );
}
#[test]
fn writable_principal_identity_is_refused_at_composition() {
    refused::<5>(
        "principal",
        FieldSelection::<Model<0>, Principal<0>, Identity<0>, u64>::at("value").aspect(),
        "value",
    );
}
#[test]
fn principal_identity_without_equality_is_refused_at_composition() {
    refused::<6>(
        "principal",
        FieldSelection::<Model<0>, Principal<0>, Identity<0>, u64>::at("value").aspect(),
        "value",
    );
}
#[test]
fn misspelled_mapping_status_is_refused_at_composition() {
    refused::<7>(
        "mapping",
        FieldSelection::<Model<0>, Mapping<0>, Status<0>, WorthQueryPrincipalMappingStatus>::at(
            "typo",
        )
        .aspect(),
        "typo",
    );
}
#[test]
fn foreign_mapping_status_aspect_is_refused_at_composition() {
    refused::<8>(
        "mapping",
        FieldSelection::<Model<0>, Mapping<0>, Status<0>, WorthQueryPrincipalMappingStatus>::at(
            "value",
        )
        .aspect(),
        "value",
    );
}
#[test]
fn foreign_principal_identity_aspect_is_refused_at_composition() {
    refused::<9>(
        "principal",
        FieldSelection::<Model<0>, Principal<0>, Identity<0>, u64>::at("value").aspect(),
        "value",
    );
}
#[test]
fn misspelled_principal_identity_is_refused_at_composition() {
    refused::<10>(
        "principal",
        FieldSelection::<Model<0>, Principal<0>, Identity<0>, u64>::at("typo").aspect(),
        "typo",
    );
}

#[test]
fn a_principal_codec_mismatch_is_refused_at_composition() {
    let expected = FieldSelection::<Model<0>, Principal<0>, Identity<0>, u64>::at("value")
        .binding_recipe()
        .locus()
        .clone();
    let denial = declaration_builder(Model::<0>)
        .unwrap()
        .principal_binding(wrong_codec_reference())
        .build()
        .unwrap_err();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::InvalidPrincipalBindingField {
            binding: "actor".into(),
            field: expected.clone()
        }
    );
    let message = denial.to_string();
    for part in [
        "actor",
        expected.entity(),
        expected.aspect(),
        expected.field(),
    ] {
        assert!(message.contains(part), "{message}");
    }
}

#[test]
fn a_malformed_principal_path_retains_its_owner_and_path() {
    refused::<11>("mapping", FieldSelection::<Model<0>, Mapping<0>, External<0>, WorthQueryExternalPrincipalIdentity>::at("value").aspect(), "bad field");
}
#[test]
fn a_principal_requirement_accepts_a_nested_record_field_path() {
    Model::<12>::declaration().unwrap();
}

#[test]
fn principal_posture_uses_the_selected_field_instead_of_its_queryable_sibling() {
    Model::<0>::declaration().unwrap();
    refused::<13>(
        "principal",
        FieldSelection::<Model<0>, Principal<0>, Identity<0>, u64>::at("unindexed").aspect(),
        "unindexed",
    );
}
