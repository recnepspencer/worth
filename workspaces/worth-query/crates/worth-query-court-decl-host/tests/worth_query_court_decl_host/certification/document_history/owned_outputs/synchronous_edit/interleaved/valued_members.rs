//! A model's members whose value equals 8 (PR C C4 r2): an incoming
//! prerequisite selection a member write changes without a link.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Valued {
    pub members: Vec<String>,
}
worth_query_structured_value_binding!(pub ResultBinding for Valued { identity: "owned-outputs.valued-members.result.v1" });
worth_query_structured_value_binding!(pub ChildBinding for String { identity: "owned-outputs.valued-members.child.v1" });
worth_query_application_query!(pub Query for Schema, identity "owned-outputs.valued-members.query.v1",
    parameters query::ParametersBinding, result ResultBinding, scope Model => "Model", name "ValuedMembers");
pub struct Members;
pub struct MembersKey;
pub struct Wanted;
worth_query_portable_type!(Members => "owned-outputs.valued-members.members");
worth_query_portable_type!(MembersKey => "owned-outputs.valued-members.members-key");
pub type ValuedMembers = ApplicationQueryResultRelationRef<
    Query,
    Members,
    Schema,
    ModelMember,
    Model,
    Member,
    ForwardResultTraversal,
    ManyResults,
>;
pub fn valued() -> ValuedMembers {
    ApplicationQueryResultRelationRef::forward_many("valued", ModelMember::reference())
}
fn members_key() -> ApplicationQueryResultFieldRef<Query, MembersKey, Schema, MemberKey> {
    ApplicationQueryResultFieldRef::new("valued-key", MemberKey::at("MemberKey"))
}
fn wanted() -> ApplicationQueryParameterRef<Query, Wanted, U64ApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("wanted")
}
fn parameters() -> ApplicationQueryParameterSet<Query> {
    ApplicationQueryParameterSet::new().bind(wanted(), 8).unwrap()
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let rows =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Member, String, ChildBinding>::new(
            Member::reference(),
        )
        .field(members_key());
    let shape =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Model, Valued, ResultBinding>::new(
            Model::reference(),
        )
        .relation_where_equal(valued(), rows, MemberValue::at("MemberValue"), wanted())
        .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Model::reference())
        .scope(Model::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(wanted());
    schema
        .application_query(definition)
        .application_query_binding::<Binding>()
}
impl WorthQueryApplicationProjection<Schema, Query> for Valued {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            members: row
                .many(valued())?
                .iter()
                .map(|row| row.field(members_key()))
                .collect::<Result<_, _>>()?,
        })
    }
}
/// One selected member row: its key.
impl WorthQueryApplicationProjection<Schema, Query> for String {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        row.field(members_key())
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub Input for Read { identity: "owned-outputs.valued-members.input.v1" });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<ModelKey>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = Input;
    type Query = Query;
    type ParameterBinding = query::ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "owned-outputs.valued-members.binding.v1";
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
        parameters()
    }
    fn into_scope(self) -> Scope {
        Scope::new(ModelKey::at("ModelKey"), self.0)
    }
}
