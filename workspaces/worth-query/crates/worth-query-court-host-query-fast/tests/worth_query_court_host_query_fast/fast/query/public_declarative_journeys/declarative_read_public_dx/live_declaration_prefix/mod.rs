pub(crate) use worth_query_worlds::facade::query::public_declarative_journeys::declarative_read_public_dx::live_declaration_prefix::*;

#[test]
fn managed_live_prefix_uses_only_the_live_capability_namespace() {
    let declaration = declare("users.current", |read| {
        read.local_collection(
            "user",
            QuerySchemaView::new(
                "public-dx-managed-live",
                [SchemaFieldView::new(
                    AspectName::new("identity").expect("aspect should build"),
                    FieldName::new("id").expect("field should build"),
                    ScalarAspectType::String,
                )],
                [],
            ),
            |query| {
                query.project(
                    AspectFieldSelector::new("identity", "id").expect("projection should build"),
                )
            },
            |shape| {
                shape.field(
                    AuthoredResultShapeField::new("identity", "id", "id")
                        .expect("result field should build"),
                )
            },
        )
    })
    .expect("managed live declaration should canonicalize");

    assert_eq!(declaration.name(), "users.current");
    assert!(!declaration.identity().as_str().is_empty());
    assert_eq!(
        declaration.using(current()).context_kind(),
        WorthQueryReadContextKind::Current
    );
}
