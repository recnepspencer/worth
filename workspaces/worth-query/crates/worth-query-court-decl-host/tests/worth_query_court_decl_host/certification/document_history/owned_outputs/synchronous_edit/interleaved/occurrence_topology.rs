//! Complete source prerequisite rows and inverse dependent discovery in the existing cabinet.
use super::*;
worth_query_application_query!(pub Query for Schema, identity "owned-outputs.occurrence-topology.v1",
    parameters query::ParametersBinding, result TopologyBinding, scope Model => "Model", name "OccurrenceTopology");
#[derive(Clone, Debug)]
pub struct Topology {
    pub parents: Vec<Parent>,
    pub children: Vec<String>,
}
worth_query_structured_value_binding!(pub TopologyBinding for Topology { identity:"owned-outputs.occurrence-topology-result.v1" });
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Parent {
    pub role: String,
    pub key: String,
}
worth_query_structured_value_binding!(pub ParentBinding for Parent { identity:"owned-outputs.occurrence-parent.v1" });
pub struct Parents;
pub struct ParentModel;
pub struct Children;
pub struct ChildModel;
pub struct Key;
pub struct Role;
worth_query_portable_type!(Parents=>"owned-outputs.occurrence-parents");
worth_query_portable_type!(ParentModel=>"owned-outputs.occurrence-parent-model");
worth_query_portable_type!(Children=>"owned-outputs.occurrence-children");
worth_query_portable_type!(ChildModel=>"owned-outputs.occurrence-child-model");
worth_query_portable_type!(Key=>"owned-outputs.occurrence-key");
worth_query_portable_type!(Role=>"owned-outputs.occurrence-role");
pub fn parents() -> ApplicationQueryResultRelationRef<
    Query,
    Parents,
    Schema,
    ModelMember,
    Model,
    Member,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("parents", ModelMember::reference())
}
fn parent() -> ApplicationQueryResultRelationRef<
    Query,
    ParentModel,
    Schema,
    UpstreamOf,
    Member,
    Model,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("parent", UpstreamOf::reference())
}
fn children() -> ApplicationQueryResultRelationRef<
    Query,
    Children,
    Schema,
    UpstreamOf,
    Member,
    Model,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many("dependents", UpstreamOf::reference())
}
fn child() -> ApplicationQueryResultRelationRef<
    Query,
    ChildModel,
    Schema,
    ModelMember,
    Model,
    Member,
    ReverseResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::reverse_one("dependent", ModelMember::reference())
}
fn key() -> ApplicationQueryResultFieldRef<Query, Key, Schema, ModelKey> {
    ApplicationQueryResultFieldRef::new("key", ModelKey::at("ModelKey"))
}
fn role() -> ApplicationQueryResultFieldRef<Query, Role, Schema, MemberKey> {
    ApplicationQueryResultFieldRef::new("role", MemberKey::at("MemberKey"))
}
impl WorthQueryApplicationProjection<Schema, Query> for Parent {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            role: row.field(role())?,
            key: row.one(parent())?.field(key())?,
        })
    }
}
impl WorthQueryApplicationProjection<Schema, Query> for Topology {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            parents: row
                .many(parents())?
                .iter()
                .map(|r| Parent::project(&r))
                .collect::<Result<_, _>>()?,
            children: row
                .many(children())?
                .iter()
                .map(|r| r.one(child())?.field(key()))
                .collect::<Result<_, _>>()?,
        })
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let model = || {
        ApplicationQueryResultShapeBuilder::<Schema, Query, Model, Topology, TopologyBinding>::new(
            Model::reference(),
        )
        .field(key())
    };
    let parent_row = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Member,
        Topology,
        TopologyBinding,
    >::new(Member::reference())
    .field(role())
    .relation(parent(), model());
    let child_row = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Member,
        Topology,
        TopologyBinding,
    >::new(Member::reference())
    .relation(child(), model());
    let definition = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Model::reference())
        .scope(Model::reference())
        .result_shape(
            model()
                .relation(parents(), parent_row)
                .relation(children(), child_row)
                .build(),
        )
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
worth_query_structured_value_binding!(pub Input for Read { identity:"owned-outputs.occurrence-topology-input.v1" });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<ModelKey>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = Input;
    type Query = Query;
    type ParameterBinding = query::ParametersBinding;
    type ResultBinding = TopologyBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "owned-outputs.occurrence-topology-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ModelKey {
        ModelKey::at("ModelKey")
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
        Scope::new(ModelKey::at("ModelKey"), self.0)
    }
}
