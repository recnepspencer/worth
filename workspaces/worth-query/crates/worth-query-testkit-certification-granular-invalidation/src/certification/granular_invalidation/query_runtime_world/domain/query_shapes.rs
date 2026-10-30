pub use std::sync::OnceLock;

pub use worth_query::facade::read;
pub use worth_query_host::facade::declaration::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        CollectionQueryBuilder, CollectionResultShapeBuilder, DetailQueryBuilder,
        DetailResultShapeBuilder, OrderingSelector, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::{canonicalize_request, CanonicalQueryBundle},
};

pub fn detail_read_declaration() -> &'static read::WorthQueryReadDeclaration {
    static DECLARATION: OnceLock<read::WorthQueryReadDeclaration> = OnceLock::new();
    DECLARATION.get_or_init(|| {
        read::declare(|builder| {
            builder.local_detail(
                "TemporalIntent",
                schema_view(),
                |query| {
                    query
                        .project(field("IntentIdentityField"))
                        .project(field("IntentGateField"))
                },
                |shape| {
                    shape
                        .field(result_field("IntentIdentityField", "identity"))
                        .field(result_field("IntentGateField", "gate"))
                },
            )
        })
        .expect("the temporal certification read must remain canonical")
    })
}

pub fn ordered_read_declaration() -> &'static read::WorthQueryReadDeclaration {
    static DECLARATION: OnceLock<read::WorthQueryReadDeclaration> = OnceLock::new();
    DECLARATION.get_or_init(|| {
        read::declare(|builder| {
            builder.local_collection(
                "TemporalIntent",
                schema_view(),
                |query| {
                    query
                        .project(field("IntentIdentityField"))
                        .project(field("IntentGateField"))
                        .order_by(
                            OrderingSelector::ascending("IntentFacts", "IntentGateField").unwrap(),
                        )
                },
                |shape| shape.field(result_field("IntentIdentityField", "identity")),
            )
        })
        .expect("the ordered certification read must remain canonical")
    })
}

pub fn detail_patch_query() -> CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(root())
        .project(field("IntentIdentityField"))
        .project(field("IntentGateField"))
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(result_field("IntentIdentityField", "identity"))
        .field(result_field("IntentGateField", "gate"))
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}

pub fn ordered_portfolio_query() -> CanonicalQueryBundle {
    let query = CollectionQueryBuilder::new(root())
        .project(field("IntentIdentityField"))
        .project(field("IntentGateField"))
        .order_by(OrderingSelector::ascending("IntentFacts", "IntentGateField").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = CollectionResultShapeBuilder::new()
        .field(result_field("IntentIdentityField", "identity"))
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}

pub fn schema_view() -> read::QuerySchemaView {
    read::QuerySchemaView::new(
        "temporal-primary",
        [
            read::SchemaFieldView::new(
                read::AspectName::new("IntentFacts").unwrap(),
                read::FieldName::new("IntentIdentityField").unwrap(),
                read::ScalarAspectType::String,
            ),
            read::SchemaFieldView::new(
                read::AspectName::new("IntentFacts").unwrap(),
                read::FieldName::new("IntentGateField").unwrap(),
                read::ScalarAspectType::String,
            ),
        ],
        [],
    )
}

pub fn root() -> RootEntityKey {
    RootEntityKey::new("TemporalIntent").unwrap()
}

pub fn field(name: &str) -> AspectFieldSelector {
    AspectFieldSelector::new("IntentFacts", name).unwrap()
}

pub fn result_field(name: &str, alias: &str) -> AuthoredResultShapeField {
    AuthoredResultShapeField::new("IntentFacts", name, alias).unwrap()
}
