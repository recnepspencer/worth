//! A producer owns its separate realization, with a typed link to immutable source.
use super::*;

pub type RealizationTarget = WorthQueryInvariantMutationTarget<Schema, Realization>;
pub struct PublicationDecision {
    pub source: Target,
    pub realization: Option<RealizationTarget>,
    pub children: Vec<Option<Target>>,
    pub retire: Vec<Retirement>,
    pub foreign: Option<RealizationTarget>,
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let schema = schema
        .application_query(realization_read::definition())
        .application_query_binding::<realization_read::Binding>();
    source_control::declare(producers::leaf::declare(producers::child::declare(
        producers::parent::declare(producers::independent_parent::declare(schema)),
    )))
}

pub fn canonical_query() -> worth_query_decl::facade::canonicalization::CanonicalQueryBundle {
    use worth_query_decl::facade::{
        authoring::*, binding::QueryBindingDescriptor, canonicalization::canonicalize_request,
    };
    let query = DetailQueryBuilder::new(RootEntityKey::new("Realization").unwrap())
        .project(
            AspectFieldSelector::new(RealizationFacts::reference().name(), "RealizedValue")
                .unwrap(),
        )
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new(
                RealizationFacts::reference().name(),
                "RealizedValue",
                "value",
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

pub use super::schema::storage_model::{
    Realization, RealizationFacts, RealizationKey, RealizedValue, Realizes,
};
