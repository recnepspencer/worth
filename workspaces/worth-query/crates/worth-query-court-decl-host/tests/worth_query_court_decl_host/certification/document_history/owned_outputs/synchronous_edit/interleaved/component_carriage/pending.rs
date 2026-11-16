//! Real incoming parents of a still-pending child, with no outgoing substitute.
use super::*;
worth_query_application_query!(pub Query for Schema, identity "owned-outputs.pending-child-parents.v1",
    parameters query::ParametersBinding, result SourceBinding, scope FirstOutput => "FirstOutput", name "PendingChildParents");
#[derive(Clone, Debug)]
pub struct Source;
#[derive(Clone, Debug)]
pub struct Parent {
    pub key: String,
}
worth_query_structured_value_binding!(pub SourceBinding for Source { identity:"owned-outputs.pending-child-source.v1" });
worth_query_structured_value_binding!(pub ParentBinding for Parent { identity:"owned-outputs.pending-child-parent-row.v1" });
pub struct Members;
pub struct Owner;
pub struct Key;
worth_query_portable_type!(Members=>"owned-outputs.pending-child-members");
worth_query_portable_type!(Owner=>"owned-outputs.pending-child-owner");
worth_query_portable_type!(Key=>"owned-outputs.pending-child-key");
type MemberRows = ApplicationQueryResultRelationRef<
    Query,
    Members,
    Schema,
    PendingChildOf,
    Member,
    FirstOutput,
    ReverseResultTraversal,
    ManyResults,
>;
type Owners = ApplicationQueryResultRelationRef<
    Query,
    Owner,
    Schema,
    ModelMember,
    Model,
    Member,
    ReverseResultTraversal,
    ExactlyOneResult,
>;
fn members() -> MemberRows {
    ApplicationQueryResultRelationRef::reverse_many("members", PendingChildOf::reference())
}
fn owner() -> Owners {
    ApplicationQueryResultRelationRef::reverse_one("owner", ModelMember::reference())
}
fn key() -> ApplicationQueryResultFieldRef<Query, Key, Schema, ModelKey> {
    ApplicationQueryResultFieldRef::new("key", ModelKey::at("ModelKey"))
}
impl WorthQueryApplicationProjection<Schema, Query> for Source {
    fn project(
        _: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self)
    }
}
impl WorthQueryApplicationProjection<Schema, Query> for Parent {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key())?,
        })
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let model =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Model, Source, SourceBinding>::new(
            Model::reference(),
        )
        .field(key());
    let member =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Member, Source, SourceBinding>::new(
            Member::reference(),
        )
        .relation(owner(), model);
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        FirstOutput,
        Source,
        SourceBinding,
    >::new(FirstOutput::reference())
    .relation(members(), member)
    .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(FirstOutput::reference())
        .scope(FirstOutput::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public();
    schema
        .application_query(definition)
        .application_query_binding::<Binding>()
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub QueryInput for Read { identity:"owned-outputs.pending-child-input.v1" });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<FirstKey>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = QueryInput;
    type Query = Query;
    type ParameterBinding = query::ParametersBinding;
    type ResultBinding = SourceBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "owned-outputs.pending-child-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> FirstKey {
        FirstKey::at("FirstKey")
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
        Scope::new(FirstKey::at("FirstKey"), self.0)
    }
}
pub struct Input;
impl ApplicationInputPort<Schema, hierarchy::Child> for Input {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "component-carriage.pending-parent-input";
    const REQUIRED: bool = false;
}
pub struct Pending;
impl ApplicationConnectionIdentity for Pending {
    const IDENTITY: &'static str = "component-carriage.pending-parent";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Child>
    for Pending
{
}
pub type Connection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Child,
    Input,
    Pending,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Pending {
    type RootDemand = Demand;
    type Demand = from_first::Demand;
    type Discovery = query::first_output::Read;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("first:{}", root.source)))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        // Static's real selector owns the children; this connection supplies
        // their additional incoming parents, never another child enumeration.
        Ok(Vec::new())
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Pending {
    type Prerequisites = Read;
    type PrerequisiteRow = ParentBinding;
    type Selection = (MemberRows, Owners);
    fn prerequisite_rows() -> Self::Selection {
        (members(), owner())
    }
    fn prerequisites_from_target(
        target: &from_first::Demand,
    ) -> Result<Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Read(target.0.clone()))
    }
    fn parent_from_prerequisite(
        row: &Parent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand {
            source: row.key.clone(),
            selector: None,
        })
    }
}
