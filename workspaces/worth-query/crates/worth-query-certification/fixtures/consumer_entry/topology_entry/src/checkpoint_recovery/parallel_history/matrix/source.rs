//! Reads the committed semantic value and application count through a public query.
use super::application::*;
use crate::principal::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_portable_type,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};
#[derive(Clone, Debug)]
pub struct Read {
    pub key: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Result {
    pub key: String,
    pub revision: u64,
}
pub struct Parameters;
worth_query_structured_value_binding!(pub InputBinding for Read {identity:"matrix.source.input"});
worth_query_structured_value_binding!(pub ParametersBinding for Parameters {identity:"matrix.source.parameters"});
worth_query_structured_value_binding!(pub ResultBinding for Result {identity:"matrix.source.result"});
pub struct Query;
pub struct KeySlot;
pub struct RevisionSlot;
worth_query_portable_type!(KeySlot=>"matrix.value.slot");
worth_query_portable_type!(RevisionSlot=>"matrix.applied.slot");
impl<S: SchemaBinding> ApplicationQueryMarkerIdentity<S> for Query {
    type ParameterBinding = ParametersBinding;
    type ResultBinding = ResultBinding;
    type Scope = Node;
    const IDENTIFIER: &'static str = "MatrixSourceQuery";
    const QUERY_TYPE_NAME: &'static str = "matrix.source.query";
    const SCOPE_TYPE_NAME: &'static str = "Node";
}
fn value<S: SchemaBinding>() -> ApplicationQueryResultFieldRef<
    Query,
    KeySlot,
    S,
    Node,
    NodeFacts,
    NodeKey,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("key", NodeKey::reference())
}
fn applied<S: SchemaBinding>() -> ApplicationQueryResultFieldRef<
    Query,
    RevisionSlot,
    S,
    Node,
    NodeFacts,
    Revision,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("revision", Revision::reference())
}
fn definition<S: SchemaBinding>() -> ApplicationQueryDefinition<S, Query, Parameters, Result, Node>
{
    let shape = ApplicationQueryResultShapeBuilder::<S, Query, Node, Result, ResultBinding>::new(
        Node::reference(),
    )
    .field(value())
    .field(applied())
    .build();
    ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::<
        S,
        Query,
        Parameters,
        Result,
        Node,
    >::from_declaration())
    .root(Node::reference())
    .scope(Node::reference())
    .result_shape(shape)
    .cardinality(ApplicationQueryCardinality::ExactlyOne)
    .disclosure(ApplicationQueryDisclosureContract::public())
    .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
    .lanes(ApplicationQueryLaneEligibility::one_shot())
    .public()
    .build()
    .unwrap()
}
impl<S: SchemaBinding> WorthQueryApplicationProjection<S, Query> for Result {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, S, Query>,
    ) -> std::result::Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(value())?,
            revision: row.field(applied())?,
        })
    }
}
pub struct Binding<S>(PhantomData<fn() -> S>);
pub type Scope<S> =
    ApplicationQueryFieldScope<S, Node, NodeFacts, NodeKey, String, ReadOnly, NoApplicationUnit>;
impl<S: SchemaBinding> ApplicationQueryBinding<S> for Binding<S> {
    type Input = Read;
    type InputBinding = InputBinding;
    type Query = Query;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = Scope<S>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "matrix.source";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        S,
        Node,
        NodeFacts,
        NodeKey,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        NodeKey::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        S,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
}
impl<S: SchemaBinding> ApplicationQueryIntent<S> for Read {
    type Binding = Binding<S>;
    fn parameters(&self) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope<S> {
        Scope::new(NodeKey::reference(), format!("member-{:03}", self.key))
    }
}
pub fn declare<S: SchemaBinding>(
    builder: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    builder
        .application_query(definition::<S>())
        .application_query_binding::<Binding<S>>()
}
