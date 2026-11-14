//! Complete source prerequisite rows and inverse dependent discovery in the existing cabinet.
use super::*;
worth_query_application_query!(pub Query for Schema, identity "owned-outputs.component-carriage-topology.v1",
    parameters query::ParametersBinding, result TopologyBinding, scope Model => "Model", name "ComponentCarriageTopology");
#[derive(Clone, Debug)]
pub struct Topology {
    pub children: Vec<String>,
}
worth_query_structured_value_binding!(pub TopologyBinding for Topology { identity:"owned-outputs.component-carriage-topology-result.v1" });
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Parent {
    pub role: String,
    pub key: String,
}
worth_query_structured_value_binding!(pub ParentBinding for Parent { identity:"owned-outputs.component-carriage-parent.v1" });
pub struct PreviousParents;
pub struct ForeignParents;
pub struct PreviousRole;
pub struct ForeignRole;
worth_query_portable_type!(PreviousParents=>"owned-outputs.component-carriage-previous-parents");
worth_query_portable_type!(ForeignParents=>"owned-outputs.component-carriage-foreign-parents");
worth_query_portable_type!(PreviousRole=>"owned-outputs.component-carriage-previous-role");
worth_query_portable_type!(ForeignRole=>"owned-outputs.component-carriage-foreign-role");
type SelectedParents<Slot> = ApplicationQueryResultRelationRef<
    Query,
    Slot,
    Schema,
    ModelMember,
    Model,
    Member,
    ForwardResultTraversal,
    ManyResults,
>;
pub fn previous_parents() -> SelectedParents<PreviousParents> {
    ApplicationQueryResultRelationRef::forward_many("previous-parents", ModelMember::reference())
}
pub fn foreign_parents() -> SelectedParents<ForeignParents> {
    ApplicationQueryResultRelationRef::forward_many("foreign-parents", ModelMember::reference())
}
fn previous_role(
) -> ApplicationQueryParameterRef<Query, PreviousRole, StringApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("previous-role")
}
fn foreign_role() -> ApplicationQueryParameterRef<Query, ForeignRole, StringApplicationValueBinding>
{
    ApplicationQueryParameterRef::from_query_identifier("foreign-role")
}
pub struct ParentModel;
pub struct Children;
pub struct ChildModel;
pub struct ParentKey;
pub struct ChildKey;
pub struct Role;
worth_query_portable_type!(ParentModel=>"owned-outputs.component-carriage-parent-model");
worth_query_portable_type!(Children=>"owned-outputs.component-carriage-children");
worth_query_portable_type!(ChildModel=>"owned-outputs.component-carriage-child-model");
worth_query_portable_type!(ParentKey=>"owned-outputs.component-carriage-parent-key");
worth_query_portable_type!(ChildKey=>"owned-outputs.component-carriage-child-key");
worth_query_portable_type!(Role=>"owned-outputs.component-carriage-role");
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
fn parent_key() -> ApplicationQueryResultFieldRef<Query, ParentKey, Schema, ModelKey> {
    ApplicationQueryResultFieldRef::new("parent-key", ModelKey::at("ModelKey"))
}
fn child_key() -> ApplicationQueryResultFieldRef<Query, ChildKey, Schema, ModelKey> {
    ApplicationQueryResultFieldRef::new("child-key", ModelKey::at("ModelKey"))
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
            key: row.one(parent())?.field(parent_key())?,
        })
    }
}
impl WorthQueryApplicationProjection<Schema, Query> for Topology {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            children: row
                .many(children())?
                .iter()
                .map(|r| r.one(child())?.field(child_key()))
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
    };
    let parent_row = || {
        ApplicationQueryResultShapeBuilder::<Schema, Query, Member, Topology, TopologyBinding>::new(
            Member::reference(),
        )
        .field(role())
        .relation(parent(), model().field(parent_key()))
    };
    let child_row = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Member,
        Topology,
        TopologyBinding,
    >::new(Member::reference())
    .relation(child(), model().field(child_key()));
    let definition = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Model::reference())
        .scope(Model::reference())
        .result_shape(
            model()
                .relation_where_equal(
                    previous_parents(),
                    parent_row(),
                    MemberKey::at("MemberKey"),
                    previous_role(),
                )
                .relation_where_equal(
                    foreign_parents(),
                    parent_row(),
                    MemberKey::at("MemberKey"),
                    foreign_role(),
                )
                .relation(children(), child_row)
                .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(previous_role())
        .parameter(foreign_role());
    schema
        .application_query(definition)
        .application_query_binding::<Binding>()
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub Input for Read { identity:"owned-outputs.component-carriage-topology-input.v1" });
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
    const IDENTITY: &'static str = "owned-outputs.component-carriage-topology-binding.v1";
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
            .bind(previous_role(), format!("{}:previous", self.0))
            .unwrap()
            .bind(foreign_role(), format!("{}:foreign", self.0))
            .unwrap()
    }
    fn into_scope(self) -> Scope {
        Scope::new(ModelKey::at("ModelKey"), self.0)
    }
}
