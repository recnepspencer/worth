//! Public reads of the model source and each actual output, with native identity.
use super::*;
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct Row {
    pub native_id: worth_relational::facade::identity::EntityId,
    pub key: String,
    pub value: u64,
}
worth_query_structured_value_binding!(pub RowBinding for Row { identity: "owned-outputs.row.v1" });
pub struct Parameters;
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "owned-outputs.parameters.v1" });
macro_rules! scalar_read { ($module:ident, $entity:ident, $facts:ident, $key:ident, $value:ident, $semantic:literal, $query:literal, $binding:literal, $input:literal, $name:literal, $authorization:ident $(, $ability:expr)?) => { pub mod $module {
use super::*;
use super::super::schema::{$entity as Entity, $key as Key, $value as Value};
worth_query_application_query!(pub Query for Schema, identity $query, parameters ParametersBinding, result RowBinding, scope Entity => $semantic, name $name);
pub struct KeySlot;
pub struct ValueSlot;
worth_query_portable_type!(KeySlot => "owned-outputs.key-slot.v1");
worth_query_portable_type!(ValueSlot => "owned-outputs.value-slot.v1");
fn key() -> ApplicationQueryResultFieldRef<Query, KeySlot, Schema, Key> {
    ApplicationQueryResultFieldRef::new("key", Key::at(stringify!($key)))
}
fn value() -> ApplicationQueryResultFieldRef<Query, ValueSlot, Schema, Value> {
    ApplicationQueryResultFieldRef::new("value", Value::at(stringify!($value)))
}
pub fn declare(schema: ApplicationSchemaDeclarationBuilder<Schema>) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let shape = ApplicationQueryResultShapeBuilder::<Schema, Query, Entity, Row, RowBinding>::new(Entity::reference())
        .field(key())
        .field(value())
        .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Entity::reference())
        .scope(Entity::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .$authorization($($ability)?);
    schema
        .application_query(definition)
        .application_query_binding::<Binding>()
        .application_query_inspection(
            ApplicationQueryInspectionDefinition::<Schema, Binding>::new(ApplicationQueryDisclosureContract::public()).unwrap(),
        )
}
impl WorthQueryApplicationProjection<Schema, Query> for Row {
    fn project(row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        super::super::source_projection::record($query);
        Ok(Self { native_id: row.entity_id(), key: row.field(key())?, value: row.field(value())? })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
impl From<String> for Read { fn from(key: String) -> Self { Self(key) } }
worth_query_structured_value_binding!(pub ReadInput for Read { identity: $input });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<Key>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = ReadInput;
    type Query = Query;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = RowBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = $binding;
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> Key {
        Key::at(stringify!($key))
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<Schema, Actor, Mapping, Principal, u64, U64ApplicationValueBinding> {
        Actor::reference()
    }
}
impl ApplicationQueryIntent<Schema> for Read {
    type Binding = Binding;
    fn parameters(&self) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope {
        Scope::new(Key::at(stringify!($key)), self.0)
    }
}
} }; }
scalar_read!(
    model,
    Model,
    ModelFacts,
    ModelKey,
    ModelValue,
    "Model",
    "owned-outputs.model-read.v1",
    "owned-outputs.model-binding.v1",
    "owned-outputs.model-input.v1",
    "Model",
    public
);
pub mod members;
pub mod producer_model;
scalar_read!(
    member,
    Member,
    MemberFacts,
    MemberKey,
    MemberValue,
    "Member",
    "owned-outputs.member-read.v1",
    "owned-outputs.member-binding.v1",
    "owned-outputs.member-input.v1",
    "Member",
    requires_ability,
    permission::ReadMember::reference()
);
scalar_read!(
    first_output,
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    "FirstOutput",
    "owned-outputs.first-read.v1",
    "owned-outputs.first-binding.v1",
    "owned-outputs.first-input.v1",
    "FirstOutput",
    public
);
scalar_read!(
    second_output,
    SecondOutput,
    SecondFacts,
    SecondKey,
    SecondValue,
    "SecondOutput",
    "owned-outputs.second-read.v1",
    "owned-outputs.second-binding.v1",
    "owned-outputs.second-input.v1",
    "SecondOutput",
    public
);
scalar_read!(
    member_public,
    Member,
    MemberFacts,
    MemberKey,
    MemberValue,
    "Member",
    "owned-outputs.member-public-query.v1",
    "owned-outputs.member-public-binding.v1",
    "owned-outputs.member-public-input.v1",
    "MemberPublicRead",
    public
);
scalar_read!(
    child_output,
    ChildOutput,
    ChildFacts,
    ChildKey,
    ChildValue,
    "ChildOutput",
    "owned-outputs.child-read.v1",
    "owned-outputs.child-binding.v1",
    "owned-outputs.child-input.v1",
    "ChildOutput",
    public
);
scalar_read!(
    first_discovery,
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    "FirstOutput",
    "owned-outputs.first-discovery-query.v1",
    "owned-outputs.first-discovery-binding.v1",
    "owned-outputs.first-discovery-input.v1",
    "FirstDiscovery",
    requires_ability,
    permission::ReadFirst::reference()
);
scalar_read!(
    first_member_discovery,
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    "FirstOutput",
    "owned-outputs.first-member-discovery-query.v1",
    "owned-outputs.first-member-discovery-binding.v1",
    "owned-outputs.first-member-discovery-input.v1",
    "FirstMemberDiscovery",
    requires_ability,
    permission::ReadFirstWithMember::reference()
);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let schema = model::declare(schema);
    let schema = members::declare(schema);
    let schema = producer_model::declare(schema);
    let schema = member::declare(schema);
    let schema = first_output::declare(schema);
    let schema = first_discovery::declare(schema);
    let schema = first_member_discovery::declare(schema);
    let schema = second_output::declare(schema);
    let schema = child_output::declare(schema);
    member_public::declare(schema)
}
