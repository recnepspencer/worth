pub use std::sync::OnceLock;

pub use worth_query::facade::{domain, read};
pub use worth_query_host::facade::declaration::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        CollectionQueryBuilder, CollectionResultShapeBuilder, EqualityPredicate, OrderingSelector,
        RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::{canonicalize_request, CanonicalQueryBundle},
};

pub fn read_declaration() -> &'static read::WorthQueryReadDeclaration {
    static DECLARATION: OnceLock<read::WorthQueryReadDeclaration> = OnceLock::new();
    DECLARATION.get_or_init(|| {
        read::declare(|builder| {
            builder.explicit_broad_search_collection(
                "MarketObservation",
                schema_view(),
                |query| {
                    query
                        .project(field("PortfolioValueField"))
                        .project(field("PortfolioDeskField"))
                        .where_equal(
                            EqualityPredicate::new("PortfolioFacts", "PortfolioDeskField", "rates")
                                .unwrap(),
                        )
                        .order_by(
                            OrderingSelector::ascending("PortfolioFacts", "PortfolioRankField")
                                .unwrap(),
                        )
                },
                |shape| {
                    shape.field(
                        AuthoredResultShapeField::new(
                            "PortfolioFacts",
                            "PortfolioValueField",
                            "position_value",
                        )
                        .unwrap(),
                    )
                },
            )
        })
        .expect("the financial ordered-portfolio read must remain canonical")
    })
}

pub fn canonical_query() -> CanonicalQueryBundle {
    let query = CollectionQueryBuilder::new(root())
        .project(field("PortfolioValueField"))
        .project(field("PortfolioDeskField"))
        .where_equal(
            EqualityPredicate::new("PortfolioFacts", "PortfolioDeskField", "rates").unwrap(),
        )
        .order_by(OrderingSelector::ascending("PortfolioFacts", "PortfolioRankField").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = CollectionResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new(
                "PortfolioFacts",
                "PortfolioValueField",
                "position_value",
            )
            .unwrap(),
        )
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}

pub fn collection_contract() -> domain::WorthQueryOperationCollectionContract {
    let desk = domain::WorthQueryOperationCollectionField::from_dotted(
        "PortfolioFacts.PortfolioDeskField",
    )
    .unwrap();
    let rank = domain::WorthQueryOperationCollectionField::from_dotted(
        "PortfolioFacts.PortfolioRankField",
    )
    .unwrap();
    domain::WorthQueryOperationCollectionContract::Collection {
        row_identity_field: desk.clone(),
        ordering_fields: vec![rank],
        grouping: domain::WorthQueryOperationGroupingContract::Grouped {
            grouping_fields: vec![desk],
        },
        window: domain::WorthQueryOperationWindowPolicy::ContinuationBounded,
        continuation: domain::WorthQueryOperationContinuationPosture::SnapshotCursor,
    }
}

pub fn schema_view() -> read::QuerySchemaView {
    read::QuerySchemaView::new(
        "financial-primary",
        [
            schema_field("PortfolioValueField", read::ScalarAspectType::UInt64),
            schema_field("PortfolioDeskField", read::ScalarAspectType::String),
            schema_field("PortfolioRankField", read::ScalarAspectType::UInt64),
        ],
        [],
    )
}

pub fn schema_field(name: &str, family: read::ScalarAspectType) -> read::SchemaFieldView {
    read::SchemaFieldView::new(
        read::AspectName::new("PortfolioFacts").unwrap(),
        read::FieldName::new(name).unwrap(),
        family,
    )
}

pub fn root() -> RootEntityKey {
    RootEntityKey::new("MarketObservation").unwrap()
}

pub fn field(name: &str) -> AspectFieldSelector {
    AspectFieldSelector::new("PortfolioFacts", name).unwrap()
}
