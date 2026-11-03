use super::*;
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Row {
    pub key: String,
    pub value: u64,
}
worth_query_structured_value_binding!(pub RowBinding for Row { identity: "owned-model.row.v1" });
pub struct Parameters;
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "owned-model.parameters.v1" });
macro_rules! scalar_read { ($module:ident, $entity:ident, $facts:ident, $key:ident, $value:ident, $semantic:literal, $query:literal, $binding:literal) => { pub mod $module {
use super::*;
use super::super::schema::{$entity as Entity, $facts as Facts, $key as Key, $value as Value};
worth_query_application_query!(pub Query for Schema, identity $query, parameters ParametersBinding, result RowBinding, scope Entity => $semantic, name $semantic);
const BINDING_ID: &str = $binding;
pub struct KeySlot;
pub struct ValueSlot;
worth_query_portable_type!(KeySlot => "owned-model.key-slot.v1");
worth_query_portable_type!(ValueSlot => "owned-model.value-slot.v1");
fn key() -> ApplicationQueryResultFieldRef<
    Query,
    KeySlot,
    Schema,
    Entity,
    Facts,
    Key,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("key", Key::reference())
}
fn value() -> ApplicationQueryResultFieldRef<
    Query,
    ValueSlot,
    Schema,
    Entity,
    Facts,
    Value,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("value", Value::reference())
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let shape =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Entity, Row, RowBinding>::new(
            Entity::reference(),
        )
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
            ApplicationQueryInspectionDefinition::<Schema, Binding>::new(
                ApplicationQueryDisclosureContract::public(),
            )
            .unwrap(),
        )
}
impl WorthQueryApplicationProjection<Schema, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key())?,
            value: row.field(value())?,
        })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub ReadInput for Read { identity: "owned-model.read-input.v1" });
pub struct Binding;
type Scope =
    ApplicationQueryFieldScope<Schema, Entity, Facts, Key, String, ReadOnly, NoApplicationUnit>;
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
    const IDENTITY: &'static str = BINDING_ID;
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Entity,
        Facts,
        Key,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        Key::reference()
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
    "owned-model.model-read.v1",
    "owned-model.model-binding.v1"
);
scalar_read!(
    member,
    Member,
    MemberFacts,
    MemberKey,
    MemberValue,
    "Member",
    "owned-model.member-read.v1",
    "owned-model.member-binding.v1"
);
