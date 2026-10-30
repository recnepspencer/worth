//! Independent public readback of every NumberFacts payload field.
use super::super::schema::*;
use super::{Parameters, ParametersBinding};
use worth_query_host::facade::{
    application_invariants::EntityId,
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub entity_id: EntityId,
    pub key: String,
    pub primary: u64,
    pub secondary: u64,
    pub read_enabled: bool,
    pub ceiling: u64,
    pub calculated: u64,
}
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "numeric-full-read.row.v1" });
worth_query_application_query!(pub FullQuery for NumericSchema, identity "numeric-full-read.query.v1",
    parameters ParametersBinding, result ResultBinding, scope Number => "Number", name "number_full");

macro_rules! result_field {
    ($name:ident, $Slot:ident, $Field:ty, $Value:ty, $Write:ty, $id:literal) => {
        pub struct $Slot;
        worth_query_portable_type!($Slot => $id);
        fn $name() -> ApplicationQueryResultFieldRef<
            FullQuery, $Slot, NumericSchema, Number, NumberFacts,
            $Field, $Value, $Write, EqualityPredicate, NoApplicationUnit,
        > {
            ApplicationQueryResultFieldRef::new(stringify!($name), <$Field>::reference())
        }
    };
}
result_field!(
    key,
    KeySlot,
    Key,
    String,
    ReadOnly,
    "numeric-full-read.key.v1"
);
result_field!(
    primary,
    PrimarySlot,
    Primary,
    u64,
    ReadWrite,
    "numeric-full-read.primary.v1"
);
result_field!(
    secondary,
    SecondarySlot,
    Secondary,
    u64,
    ReadWrite,
    "numeric-full-read.secondary.v1"
);
result_field!(
    read_enabled,
    ReadEnabledSlot,
    ReadEnabled,
    bool,
    ReadWrite,
    "numeric-full-read.enabled.v1"
);
result_field!(
    ceiling,
    CeilingSlot,
    Ceiling,
    u64,
    ReadWrite,
    "numeric-full-read.ceiling.v1"
);
result_field!(
    calculated,
    CalculatedSlot,
    Calculated,
    u64,
    ReadWrite,
    "numeric-full-read.calculated.v1"
);

pub fn definition() -> ApplicationQueryDefinition<NumericSchema, FullQuery, Parameters, Row, Number>
{
    let shape = ApplicationQueryResultShapeBuilder::<
        NumericSchema,
        FullQuery,
        Number,
        Row,
        ResultBinding,
    >::new(Number::reference())
    .field(key())
    .field(primary())
    .field(secondary())
    .field(read_enabled())
    .field(ceiling())
    .field(calculated())
    .build();
    ApplicationQueryDefinitionBuilder::declare(FullQuery::reference())
        .root(Number::reference())
        .scope(Number::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<NumericSchema, FullQuery> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, NumericSchema, FullQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            entity_id: row.entity_id(),
            key: row.field(key())?,
            primary: row.field(primary())?,
            secondary: row.field(secondary())?,
            read_enabled: row.field(read_enabled())?,
            ceiling: row.field(ceiling())?,
            calculated: row.field(calculated())?,
        })
    }
}

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Read {
    pub key: String,
}
worth_query_structured_value_binding!(pub InputBinding for Read { identity: "numeric-full-read.input.v1" });
type Scope = ApplicationQueryFieldScope<
    NumericSchema,
    Number,
    NumberFacts,
    Key,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
pub struct Binding;
impl ApplicationQueryBinding<NumericSchema> for Binding {
    type Input = Read;
    type InputBinding = InputBinding;
    type Query = FullQuery;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = ActorBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "numeric-full-read.binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        NumericSchema,
        Number,
        NumberFacts,
        Key,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        Key::reference()
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
impl ApplicationQueryIntent<NumericSchema> for Read {
    type Binding = Binding;
    fn parameters(&self) -> ApplicationQueryParameterSet<FullQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope {
        Scope::new(Key::reference(), self.key)
    }
}
