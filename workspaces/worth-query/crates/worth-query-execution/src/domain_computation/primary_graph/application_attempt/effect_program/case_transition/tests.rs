//! Private effect validation uses only declarations and effect vocabulary.
use super::*;
use worth_query_declaration::facade::application_schema::*;
use worth_query_testkit_decl::fragment_fixture::*;
use worth_relational::facade::identity::PartitionId;

#[test]
fn a_transition_does_not_absorb_an_independently_authored_parent_clear() {
    let declaration = declare(Fixture).unwrap();
    let fragment = declaration
        .member_provenance()
        .fragment_catalog()
        .fragment(Fixture)
        .unwrap();
    let cases = [
        fragment.aspect::<Item, Measured>().unwrap(),
        fragment.aspect::<Item, Named>().unwrap(),
    ];
    let (identity, revision) = declaration
        .erased()
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::Aspect {
                aspect,
                identity,
                revision,
                ..
            } if aspect == "measured" => Some((*identity, *revision)),
            _ => None,
        })
        .unwrap();
    let id = EntityId::new(PartitionId::main(), 1, 0);
    let effects = [Effect::RemoveAspect {
        entity: "item".to_owned(),
        entity_id: id,
        contract: PortableAspectContractBasis::new(
            AspectKey::new("measured").unwrap(),
            identity,
            revision,
        ),
        fields: Vec::new(),
    }];
    assert_eq!(
        check_case_slots(&effects, id, "item", &cases, &[])
            .unwrap_err()
            .kind(),
        Kind::ConflictingEffectStep
    );
    check_case_slots(&effects, id, "item", &cases, &[0]).unwrap();
    assert_eq!(
        check_case_slots(&effects, id, "item", &cases, &[1])
            .unwrap_err()
            .kind(),
        Kind::ConflictingEffectStep
    );
}

#[test]
fn unknown_encoded_columns_are_refused_before_any_effect_is_authored() {
    let declaration = declare(Fixture).unwrap();
    let fragment = declaration
        .member_provenance()
        .fragment_catalog()
        .fragment(Fixture)
        .unwrap();
    let aspect = fragment.aspect::<Item, Named>().unwrap();
    let mut fields = <Bindings as Record<Caption>>::record()
        .encode(
            &Caption {
                text: "value".to_owned(),
            },
            aspect.path(),
        )
        .unwrap();
    let path = format!("{}.unknown", aspect.path());
    fields.insert(
        path.clone(),
        U64ApplicationValueBinding::encode(&9u64).unwrap(),
    );
    let denial = prepare_fields(aspect, fields).unwrap_err();
    assert_eq!(denial.kind(), Kind::InvalidEffectValue);
    assert_eq!(denial.subject(), path);
}
