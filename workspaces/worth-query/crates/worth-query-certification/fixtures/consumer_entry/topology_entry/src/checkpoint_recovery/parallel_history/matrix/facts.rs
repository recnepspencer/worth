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
    pub value: u64,
    pub applied: u64,
}
pub struct Parameters;
worth_query_structured_value_binding!(pub InputBinding for Read {identity:"matrix.read.input"});
worth_query_structured_value_binding!(pub ParametersBinding for Parameters {identity:"matrix.read.parameters"});
worth_query_structured_value_binding!(pub ResultBinding for Result {identity:"matrix.read.result"});
pub struct Query;
pub struct ValueSlot;
pub struct AppliedSlot;
worth_query_portable_type!(ValueSlot=>"matrix.value.slot");
worth_query_portable_type!(AppliedSlot=>"matrix.applied.slot");
impl<S: SchemaBinding> ApplicationQueryMarkerIdentity<S> for Query {
    type ParameterBinding = ParametersBinding;
    type ResultBinding = ResultBinding;
    type Scope = Node;
    const IDENTIFIER: &'static str = "NeutralQuery";
    const QUERY_TYPE_NAME: &'static str = "matrix.query";
    const SCOPE_TYPE_NAME: &'static str = "Node";
}
fn value<S: SchemaBinding>() -> ApplicationQueryResultFieldRef<Query, ValueSlot, S, Value> {
    ApplicationQueryResultFieldRef::new("value", Value::at("Value"))
}
fn applied<S: SchemaBinding>() -> ApplicationQueryResultFieldRef<Query, AppliedSlot, S, Applied> {
    ApplicationQueryResultFieldRef::new("applied", Applied::at("Applied"))
}
fn definition<S: SchemaBinding>(
) -> ApplicationQueryDefinitionBuilder<S, Query, Parameters, Result, Node> {
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
}
impl<S: SchemaBinding> WorthQueryApplicationProjection<S, Query> for Result {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, S, Query>,
    ) -> std::result::Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            value: row.field(value())?,
            applied: row.field(applied())?,
        })
    }
}
pub struct Binding<S>(PhantomData<fn() -> S>);
pub type Scope = ApplicationQueryFieldScope<NodeKey>;
impl<S: SchemaBinding> ApplicationQueryBinding<S> for Binding<S> {
    type Input = Read;
    type InputBinding = InputBinding;
    type Query = Query;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "matrix.read";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> NodeKey {
        NodeKey::at("NodeKey")
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
    fn into_scope(self) -> Scope {
        Scope::new(NodeKey::at("NodeKey"), format!("member-{:03}", self.key))
    }
}
pub fn declare<S: SchemaBinding>(
    builder: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    builder
        .application_query(definition::<S>())
        .application_query_binding::<Binding<S>>()
}
