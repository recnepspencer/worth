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
macro_rules! scalar_read { ($module:ident, $entity:ident, $facts:ident, $key:ident, $value:ident, $semantic:literal, $query:literal, $binding:literal, $input:literal) => { pub mod $module {
use super::*;
use super::super::schema::{$entity as Entity, $facts as Facts, $key as Key, $value as Value};
worth_query_application_query!(pub Query for Schema, identity $query, parameters ParametersBinding, result RowBinding, scope Entity => $semantic, name $semantic);
pub struct KeySlot;
pub struct ValueSlot;
worth_query_portable_type!(KeySlot => "owned-outputs.key-slot.v1");
worth_query_portable_type!(ValueSlot => "owned-outputs.value-slot.v1");
fn key() -> ApplicationQueryResultFieldRef<Query, KeySlot, Schema, Entity, Facts, Key, String, ReadOnly, EqualityPredicate, NoApplicationUnit> {
    ApplicationQueryResultFieldRef::new("key", Key::reference())
}
fn value() -> ApplicationQueryResultFieldRef<Query, ValueSlot, Schema, Entity, Facts, Value, u64, ReadWrite, EqualityPredicate, NoApplicationUnit> {
    ApplicationQueryResultFieldRef::new("value", Value::reference())
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
        .public()
        .build()
        .unwrap();
    schema
        .application_query(definition)
        .application_query_binding::<Binding>()
        .application_query_inspection(
            ApplicationQueryInspectionDefinition::<Schema, Binding>::new(ApplicationQueryDisclosureContract::public()).unwrap(),
        )
}
impl WorthQueryApplicationProjection<Schema, Query> for Row {
    fn project(row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self { native_id: row.entity_id(), key: row.field(key())?, value: row.field(value())? })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub ReadInput for Read { identity: $input });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<Schema, Entity, Facts, Key, String, ReadOnly, NoApplicationUnit>;
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
    fn scope_field() -> ApplicationFieldRef<Schema, Entity, Facts, Key, String, ReadOnly, EqualityPredicate, NoApplicationUnit> {
        Key::reference()
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
        Scope::new(Key::reference(), self.0)
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
    "owned-outputs.model-input.v1"
);
scalar_read!(
    member,
    Member,
    MemberFacts,
    MemberKey,
    MemberValue,
    "Member",
    "owned-outputs.member-read.v1",
    "owned-outputs.member-binding.v1",
    "owned-outputs.member-input.v1"
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
    "owned-outputs.first-input.v1"
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
    "owned-outputs.second-input.v1"
);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    second_output::declare(first_output::declare(member::declare(model::declare(
        schema,
    ))))
}
