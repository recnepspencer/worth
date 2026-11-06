use worth_query_testkit_decl::fragment_fixture::*;
use worth_query_testkit_decl::fragment_relation_fixture::*;

#[test]
fn relation_records_lower_arrays_and_nested_fields_without_entity_ownership() {
    let declaration = declare(Fixture).unwrap();
    let catalog = declaration.member_provenance().fragment_catalog();
    let _: ApplicationRelationRef<Fixture, Connect, Origin, Origin> =
        relation(Linked, Origin, Connect);
    let field: RelationFieldSelection<_, _, _, _, u64> =
        select_relation_field(Linked, Origin, Connect, EdgeMeasurements, "interval.end");
    let column = field.installed(catalog).unwrap();
    assert_eq!(
        column.path(),
        "fixture.linked.origin.connect.measurements.interval.end"
    );
    assert_eq!(
        column.recipe().locus().owner_kind(),
        ApplicationFieldBindingOwnerKind::Relation
    );
    assert_eq!(field.checked().unwrap().recipe(), column.recipe());
    let definition = catalog
        .fragment(Linked)
        .unwrap()
        .relation_aspect::<Connect, EdgeMeasurements>()
        .unwrap();
    assert_eq!(
        definition
            .fields()
            .iter()
            .map(|field| field.recipe().locus().field())
            .collect::<Vec<_>>(),
        [
            "components[0]",
            "components[1]",
            "components[2]",
            "interval.start",
            "interval.end"
        ]
    );
    let wrong: RelationFieldSelection<_, _, _, _, bool> =
        select_relation_field(Linked, Origin, Connect, EdgeMeasurements, "interval.end");
    assert!(matches!(
        wrong.installed(catalog),
        Err(ApplicationSchemaDeclarationDenial::InvalidRecordField { .. })
    ));
    assert!(matches!(
        wrong.checked(),
        Err(ApplicationSchemaDeclarationDenial::InvalidRecordField { .. })
    ));
}

#[test]
fn history_contract_selection_retains_the_actual_relation_record_owner() {
    let builder = declaration_builder(Fixture).unwrap();
    let selected = relation_aspect(Linked, Origin, Connect, EdgeMeasurements);
    let actual = builder.selected_aspect_contract(selected).unwrap();
    let declaration = builder.build().unwrap();
    let parts = declaration
        .erased()
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::RelationAspect { definition }
                if definition.parts().relation == "connect" =>
            {
                Some(definition.parts())
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(actual, (parts.identity, parts.revision));
}
struct Other;
impl Fragment for Other {
    const NAME: &'static str = "other";
    type Bindings = <Linked as Fragment>::Bindings;
    type Members = (Origin,);
}
#[test]
fn history_contract_selection_refuses_a_different_fragment_with_the_same_native_names() {
    let builder = ApplicationSchemaDeclarationBuilder::<Fixture>::from_fragment(Other).unwrap();
    assert!(matches!(
        builder.selected_aspect_contract(relation_aspect(
            Linked,
            Origin,
            Connect,
            EdgeMeasurements
        )),
        Err(ApplicationSchemaDeclarationDenial::InvalidRecordMember { .. })
    ));
}

#[test]
fn history_contract_selection_accepts_an_extension_relation_payload() {
    use worth_query_testkit_decl::fragment_extension_fixture::{
        Annotates, Annotations, EdgeWeight,
    };
    let builder = declaration_builder(Fixture).unwrap();
    let actual = builder
        .selected_aspect_contract(relation_aspect(Annotations, Item, Annotates, EdgeWeight))
        .unwrap();
    let declaration = builder.build().unwrap();
    let parts = declaration
        .erased()
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::RelationAspect { definition }
                if definition.parts().relation == "annotates" =>
            {
                Some(definition.parts())
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(actual, (parts.identity, parts.revision));
}
