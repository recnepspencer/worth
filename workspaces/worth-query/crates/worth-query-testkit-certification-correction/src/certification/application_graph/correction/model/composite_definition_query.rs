//! Current authored inputs for the installed temporal definition.
use super::composite_definition_model::{DefinitionRevision, DefinitionRevisionFacts};
use super::schema::*;
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};
pub struct Parameters;
#[derive(Clone, Debug)]
pub struct Row {
    pub key: String,
    pub revision: u64,
    pub due: u64,
    pub lifecycle: u64,
}
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "numeric-definition.parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "numeric-definition.row.v1" });
worth_query_application_query!(pub ReadQuery for NumericSchema, identity "numeric-definition.read.v1", parameters ParametersBinding, result ResultBinding, scope Number => "Number", name "definition_input");
pub struct KeySlot;
pub struct RevisionSlot;
pub struct DueSlot;
pub struct LifecycleSlot;
worth_query_portable_type!(KeySlot => "numeric-definition.key-slot.v1");
worth_query_portable_type!(RevisionSlot => "numeric-definition.revision-slot.v1");
worth_query_portable_type!(DueSlot => "numeric-definition.due-slot.v1");
worth_query_portable_type!(LifecycleSlot => "numeric-definition.lifecycle-slot.v1");
pub fn key_field() -> ApplicationQueryResultFieldRef<ReadQuery, KeySlot, NumericSchema, Key> {
    ApplicationQueryResultFieldRef::new("key", Key::at("Key"))
}
pub fn revision_field(
) -> ApplicationQueryResultFieldRef<ReadQuery, RevisionSlot, NumericSchema, DefinitionRevision> {
    ApplicationQueryResultFieldRef::new("revision", DefinitionRevision::at("DefinitionRevision"))
}
pub fn due_field() -> ApplicationQueryResultFieldRef<ReadQuery, DueSlot, NumericSchema, Secondary> {
    ApplicationQueryResultFieldRef::new("due", Secondary::at("Secondary"))
}
pub fn lifecycle_field(
) -> ApplicationQueryResultFieldRef<ReadQuery, LifecycleSlot, NumericSchema, Ceiling> {
    ApplicationQueryResultFieldRef::new("lifecycle", Ceiling::at("Ceiling"))
}
pub fn definition(
) -> ApplicationQueryDefinitionBuilder<NumericSchema, ReadQuery, Parameters, Row, Number> {
    let shape = ApplicationQueryResultShapeBuilder::<
        NumericSchema,
        ReadQuery,
        Number,
        Row,
        ResultBinding,
    >::new(Number::reference())
    .field(key_field())
    .field(revision_field())
    .field(due_field())
    .field(lifecycle_field())
    .build();
    ApplicationQueryDefinitionBuilder::declare(ReadQuery::reference())
        .root(Number::reference())
        .scope(Number::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}
impl WorthQueryApplicationProjection<NumericSchema, ReadQuery> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, NumericSchema, ReadQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key_field())?,
            revision: row.field(revision_field())?,
            due: row.field(due_field())?,
            lifecycle: row.field(lifecycle_field())?,
        })
    }
}
pub struct Binding;
impl ApplicationQueryBinding<NumericSchema> for Binding {
    type Input = super::query::Read;
    type InputBinding = super::query::ReadInputBinding;
    type Query = ReadQuery;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = ApplicationQueryFieldScope<Key>;
    type PrincipalBinding = ActorBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "numeric-definition.read-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> Key {
        Key::at("Key")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        NumericSchema,
        ActorBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ActorBinding::reference()
    }
}
