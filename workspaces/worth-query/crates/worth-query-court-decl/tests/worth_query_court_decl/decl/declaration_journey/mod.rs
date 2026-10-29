pub(crate) use worth_query_testkit_decl::facade::decl::declaration_journey::*;

#[test]
fn declaration_facade_carries_authored_intent_into_schema_validation() {
    let query =
        authoring::RawAuthoredQuery::detail_builder(authoring::RootEntityKey::new("task").unwrap())
            .project(authoring::AspectFieldSelector::new("title", "text").unwrap())
            .build()
            .unwrap();
    let shape = authoring::RawAuthoredResultShape::detail_builder()
        .field(authoring::AuthoredResultShapeField::new("title", "text", "title").unwrap())
        .build()
        .unwrap();
    let request = authoring::GuidedAuthoringPath::pair_detail(query, shape).unwrap();

    let canonical = canonicalization::canonicalize_request(request).unwrap();
    let canonical_authority = canonical.query().authority();
    let schema = schema_view::QuerySchemaView::new(
        "task-schema-v1",
        [schema_view::SchemaFieldView::new(
            authoring::AspectName::new("title").unwrap(),
            authoring::FieldName::new("text").unwrap(),
            typed::ScalarAspectType::String,
        )],
        [],
    );
    let schema_authority = schema.basis_authority();

    let validated = validation::validate_canonical_bundle(canonical, schema).unwrap();

    assert_eq!(validated.query().canonical_authority(), canonical_authority);
    assert_eq!(validated.query().schema_basis_authority(), schema_authority);
    assert_eq!(validated.result_shape().bindings().len(), 1);
}
