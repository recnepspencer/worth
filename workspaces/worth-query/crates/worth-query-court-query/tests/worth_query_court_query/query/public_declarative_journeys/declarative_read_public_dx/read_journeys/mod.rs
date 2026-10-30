pub(crate) use worth_query_testkit_query::facade::query::public_declarative_journeys::declarative_read_public_dx::read_journeys::*;

#[test]
fn read_declaration_uses_only_the_read_capability_namespace() {
    let declaration = declare(|read| {
        read.local_detail(
            "user",
            QuerySchemaView::new(
                "public-dx-read",
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
    .expect("declaration should canonicalize");

    assert!(!declaration.identity().as_str().is_empty());
    let request = declaration.using(current());
    assert_eq!(request.context_kind(), WorthQueryReadContextKind::Current);
}

#[test]
fn ordinary_facade_converges_direct_scope_and_template_authoring() {
    let direct = declare(|read| {
        read.local_detail(
            "user",
            composition_schema(),
            |query| query.project(identity_selector()).project(name_selector()),
            |shape| shape.field(identity_field()).field(name_field()),
        )
    })
    .expect("direct declaration should canonicalize");

    let scoped = declare(|read| {
        read.local_detail_scoped(
            identity_query(),
            composition_shape(),
            composition_schema(),
            [QueryScopeDescriptor::projection(
                "display-name",
                [name_selector()],
            )],
        )
    })
    .expect("scoped declaration should canonicalize");

    let slot = TemplateParameterSlot::projection("display-name");
    let template = QueryTemplateDescriptor::detail(identity_query(), composition_shape())
        .with_slot(slot.clone());
    let bindings = TemplateBindingSet::new().bind_projection(&slot, name_selector());
    let templated =
        declare(|read| read.local_detail_template(template, bindings, composition_schema()))
            .expect("template declaration should canonicalize");

    for declaration in [&scoped, &templated] {
        assert_eq!(
            declaration.identity().canonical_query_digest(),
            direct.identity().canonical_query_digest()
        );
        assert_eq!(
            declaration.identity().canonical_result_shape_digest(),
            direct.identity().canonical_result_shape_digest()
        );
    }
}
