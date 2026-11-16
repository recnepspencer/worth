//! Persisted prerequisite rows and reverse dependent discovery for matrix occurrences.
mod occurrences;
use super::application::*;
use crate::principal::*;
pub use occurrences::Connection;
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
pub struct Read(pub u64);
#[derive(Clone, Debug)]
pub struct Topology {
    pub children: Vec<u64>,
}
#[derive(Clone, Debug)]
pub struct Parent {
    pub key: String,
}
pub struct Parameters;
worth_query_structured_value_binding!(pub InputBinding for Read {identity:"matrix.topology.input"});
worth_query_structured_value_binding!(pub ParametersBinding for Parameters {identity:"matrix.topology.parameters"});
worth_query_structured_value_binding!(pub ResultBinding for Topology {identity:"matrix.topology.result"});
worth_query_structured_value_binding!(pub ParentBinding for Parent {identity:"matrix.topology.parent"});
pub struct Query;
pub struct Parents;
pub struct Children;
pub struct ParentKey;
pub struct ChildKey;
worth_query_portable_type!(Parents=>"matrix.topology.parents");
worth_query_portable_type!(Children=>"matrix.topology.children");
worth_query_portable_type!(ParentKey=>"matrix.topology.parent-key");
worth_query_portable_type!(ChildKey=>"matrix.topology.child-key");
impl<S: SchemaBinding> ApplicationQueryMarkerIdentity<S> for Query {
    type ParameterBinding = ParametersBinding;
    type ResultBinding = ResultBinding;
    type Scope = Node;
    const IDENTIFIER: &'static str = "MatrixTopologyQuery";
    const QUERY_TYPE_NAME: &'static str = "matrix.topology.query";
    const SCOPE_TYPE_NAME: &'static str = "Node";
}
pub fn parents<S: SchemaBinding>() -> ApplicationQueryResultRelationRef<
    Query,
    Parents,
    S,
    Prerequisite,
    Node,
    Node,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("parents", Prerequisite::reference())
}
fn children<S: SchemaBinding>() -> ApplicationQueryResultRelationRef<
    Query,
    Children,
    S,
    Prerequisite,
    Node,
    Node,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many("children", Prerequisite::reference())
}
fn parent_key<S: SchemaBinding>() -> ApplicationQueryResultFieldRef<Query, ParentKey, S, NodeKey> {
    ApplicationQueryResultFieldRef::new("parent-key", NodeKey::at("NodeKey"))
}
fn child_key<S: SchemaBinding>() -> ApplicationQueryResultFieldRef<Query, ChildKey, S, NodeKey> {
    ApplicationQueryResultFieldRef::new("child-key", NodeKey::at("NodeKey"))
}
impl<S: SchemaBinding> WorthQueryApplicationProjection<S, Query> for Parent {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, S, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(parent_key())?,
        })
    }
}
impl<S: SchemaBinding> WorthQueryApplicationProjection<S, Query> for Topology {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, S, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            children: row
                .many(children())?
                .iter()
                .map(|r| r.field(child_key()).map(|value: String| member_key(&value)))
                .collect::<Result<_, _>>()?,
        })
    }
}
pub fn member_key(key: &str) -> u64 {
    key.strip_prefix("member-").unwrap().parse().unwrap()
}
pub fn declare<S: SchemaBinding>(
    builder: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let node = || {
        ApplicationQueryResultShapeBuilder::<S, Query, Node, Topology, ResultBinding>::new(
            Node::reference(),
        )
    };
    let shape = node()
        .relation(parents(), node().field(parent_key()))
        .relation(children(), node().field(child_key()))
        .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::<
        S,
        Query,
        Parameters,
        Topology,
        Node,
    >::from_declaration())
    .root(Node::reference())
    .scope(Node::reference())
    .result_shape(shape)
    .cardinality(ApplicationQueryCardinality::ExactlyOne)
    .disclosure(ApplicationQueryDisclosureContract::public())
    .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
    .lanes(ApplicationQueryLaneEligibility::one_shot())
    .public();
    builder
        .application_query(definition)
        .application_query_binding::<Binding<S>>()
}
pub struct Binding<S>(PhantomData<fn() -> S>);
type Scope = ApplicationQueryFieldScope<NodeKey>;
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
    const IDENTITY: &'static str = "matrix.topology";
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
        Scope::new(NodeKey::at("NodeKey"), format!("member-{:03}", self.0))
    }
}
