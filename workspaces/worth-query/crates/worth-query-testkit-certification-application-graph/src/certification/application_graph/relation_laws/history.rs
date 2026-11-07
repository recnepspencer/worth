//! A facade-only audit of actual disclosed entity and physical relation generations.
pub use super::schema::*;
pub use worth_query_decl::facade::{
    application_query::*, worth_query_application_query, worth_query_portable_type,
    worth_query_structured_value_binding,
};
pub use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

pub type Schema = GraphSchema<Payload>;
#[derive(Clone, Debug)]
pub struct ReadInput {
    pub key: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub key: u64,
    pub children: Vec<u64>,
}
worth_query_portable_type!(Row => "worth.query.record-history.node-row.v1");
worth_query_structured_value_binding!(pub InputBinding for ReadInput { identity: "worth.query.record-history.node-input.v1" });
worth_query_structured_value_binding!(pub Parameters for () { identity: "worth.query.record-history.node-parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "worth.query.record-history.node-row.v1" });
worth_query_structured_value_binding!(pub ChildResultBinding for u64 { identity: "worth.query.record-history.node-child.v1" });
worth_query_application_query!(pub Query for Schema, identity "RecordHistorySubjects", parameters Parameters,
    result ResultBinding, scope GraphNode<Schema> => "Node", name "record_history_subjects");
pub struct KeySlot;
pub struct ChildKeySlot;
pub struct Children;
worth_query_portable_type!(KeySlot => "worth.query.record-history.node-key.v1");
worth_query_portable_type!(ChildKeySlot => "worth.query.record-history.child-key.v1");
worth_query_portable_type!(Children => "worth.query.record-history.children.v1");

pub type Field<Slot> = ApplicationQueryResultFieldRef<Query, Slot, Schema, GraphKey<Schema>>;
pub fn key() -> Field<KeySlot> {
    ApplicationQueryResultFieldRef::new("key", node_key::<Schema>())
}
pub fn child_key() -> Field<ChildKeySlot> {
    ApplicationQueryResultFieldRef::new("child_key", node_key::<Schema>())
}
pub fn children() -> ApplicationQueryResultRelationRef<
    Query,
    Children,
    Schema,
    GraphPrecedes<Schema>,
    GraphNode<Schema>,
    GraphNode<Schema>,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("children", precedes::<Schema>())
}

pub struct Binding;
pub type Scope = ApplicationQueryFieldScope<GraphKey<Schema>>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = ReadInput;
    type InputBinding = InputBinding;
    type Query = Query;
    type ParameterBinding = Parameters;
    type ResultBinding = ResultBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = GraphPrincipal;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.record-history.node-read.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> GraphKey<Schema> {
        node_key::<Schema>()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        GraphPrincipal,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        GraphPrincipal::reference()
    }
}

impl WorthQueryApplicationProjection<Schema, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key())?,
            children: row
                .many(children())?
                .iter()
                .map(|child| child.field(child_key()))
                .collect::<Result<_, _>>()?,
        })
    }
}

pub struct Entities;
pub struct Relations;
worth_query_portable_type!(Entities => "worth.query.record-history.entities.v1");
worth_query_portable_type!(Relations => "worth.query.record-history.relations.v1");
impl ApplicationHistoryQueryBinding<Schema> for Entities {
    type SubjectRead = Binding;
    const IDENTITY: &'static str = "NodeChanges";
}
impl ApplicationHistoryQueryBinding<Schema> for Relations {
    type SubjectRead = Binding;
    const IDENTITY: &'static str = "PrecedesChanges";
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let nested = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        GraphNode<Schema>,
        u64,
        ChildResultBinding,
    >::new(node::<Schema>())
    .field(child_key());
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        GraphNode<Schema>,
        Row,
        ResultBinding,
    >::new(node::<Schema>())
    .field(key())
    .relation(children(), nested)
    .build();
    let query = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(node::<Schema>())
        .scope(node::<Schema>())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .order_by(child_key(), ApplicationQueryOrderingDirection::Descending);
    let entities = ApplicationHistoryQueryDefinition::<Schema, Entities>::entity(
        node::<Schema>(),
        ApplicationQueryDisclosureContract::public(),
    )
    .aspect(aspect(PayloadRecords, PayloadNode, NodeFacts), &schema)
    .unwrap()
    .finish()
    .unwrap();
    let relations = ApplicationHistoryQueryDefinition::<Schema, Relations>::relation(
        children(),
        ApplicationQueryDisclosureContract::public(),
    )
    .aspect(
        relation_aspect(PayloadRecords, PayloadNode, PayloadPrecedes, EdgeFacts),
        &schema,
    )
    .unwrap()
    .finish()
    .unwrap();
    filtered::declare(schema)
        .application_query(query)
        .application_query_binding::<Binding>()
        .application_history_query(entities)
        .application_history_query(relations)
}
#[path = "history/access.rs"]
pub mod access;
#[path = "history/court.rs"]
pub mod court;

#[path = "history/custody.rs"]
pub mod custody;
#[path = "history/filtered.rs"]
pub mod filtered;
