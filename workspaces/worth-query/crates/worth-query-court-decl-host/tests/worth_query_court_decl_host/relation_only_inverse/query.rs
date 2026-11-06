//! A node is live while its id resolves; this query is also the disclosure read.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Row {
    pub id: u64,
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read {
    pub id: u64,
}
pub struct Parameters;
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "relation-only-inverse.parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "relation-only-inverse.row.v1" });
worth_query_structured_value_binding!(pub InputBinding for Read { identity: "relation-only-inverse.read-input.v1" });
worth_query_application_query!(pub Query for Schema, identity "relation-only-inverse.read.v1", parameters ParametersBinding, result ResultBinding, scope Node => "Node", name "node");
pub struct IdSlot;
worth_query_portable_type!(IdSlot => "relation-only-inverse.id-slot.v1");
fn id() -> ApplicationQueryResultFieldRef<Query, IdSlot, Schema, Id> {
    ApplicationQueryResultFieldRef::new("id", Id::at("Id"))
}
pub fn definition() -> ApplicationQueryDefinition<Schema, Query, Parameters, Row, Node> {
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Node::reference())
        .scope(Node::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::new(Node::reference())
                .field(id())
                .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<Schema, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            id: row.field(id())?,
        })
    }
}
pub struct Binding;
type Scope = ApplicationQueryFieldScope<Id>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = InputBinding;
    type Query = Query;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "relation-only-inverse.read-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> Id {
        Id::at("Id")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        Actor,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        Actor::reference()
    }
}
impl ApplicationQueryIntent<Schema> for Read {
    type Binding = Binding;
    fn parameters(&self) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope {
        Scope::new(Id::at("Id"), self.id)
    }
}
