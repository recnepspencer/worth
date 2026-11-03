//! A producer owns its separate realization, with a typed link to immutable source.
use super::*;
worth_query_entity!(pub Realization for Schema);
worth_query_aspect!(pub RealizationFacts for Schema, Realization; identity = AspectIdentity(0x9176A105), revision = AspectContractRevision(1));
worth_query_field!(pub RealizationKey for Schema, Realization, RealizationFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub RealizedValue for Schema, Realization, RealizationFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_relation!(pub Realizes in Schema, Realization => Node; integrity = same_context_unbounded_retain_dangling);
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
        .entity(Realization::reference())
        .aspect(Realization::reference(), RealizationFacts::reference())
        .unique_field(Realization::reference(), RealizationKey::reference())
        .field(Realization::reference(), RealizedValue::reference())
        .relation(
            Realizes::reference(),
            Realization::reference(),
            Node::reference(),
        )
        .application_query(realization_read::definition())
        .application_query_binding::<realization_read::Binding>();
    source_control::declare(producers::leaf::declare(producers::child::declare(
        producers::parent::declare(schema),
    )))
}

pub fn canonical_query() -> worth_query_decl::facade::canonicalization::CanonicalQueryBundle {
    use worth_query_decl::facade::{
        authoring::*, binding::QueryBindingDescriptor, canonicalization::canonicalize_request,
    };
    let query = DetailQueryBuilder::new(RootEntityKey::new("Realization").unwrap())
        .project(AspectFieldSelector::new("RealizationFacts", "RealizedValue").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("RealizationFacts", "RealizedValue", "value").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
