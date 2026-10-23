//! Current numeric rows; a filtered-out subject cannot disclose its correction history.
use super::schema::*;
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};
pub struct Parameters;
pub struct ReadParameter;
pub struct PrimarySlot;
pub struct SecondarySlot;
worth_query_portable_type!(PrimarySlot => "numeric-correction.primary-slot.v1");
worth_query_portable_type!(SecondarySlot => "numeric-correction.secondary-slot.v1");
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub primary: u64,
    pub secondary: u64,
}
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "numeric-correction.parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "numeric-correction.row.v1" });
worth_query_application_query!(pub ReadQuery for NumericSchema, identity "numeric-correction.read.v1",
    parameters ParametersBinding, result ResultBinding, scope Number => "Number", name "number");
fn read_parameter(
) -> ApplicationQueryParameterRef<ReadQuery, ReadParameter, BoolApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("read_enabled")
}
fn primary_field() -> ApplicationQueryResultFieldRef<
    ReadQuery,
    PrimarySlot,
    NumericSchema,
    Number,
    NumberFacts,
    Primary,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("primary", Primary::reference())
}
fn secondary_field() -> ApplicationQueryResultFieldRef<
    ReadQuery,
    SecondarySlot,
    NumericSchema,
    Number,
    NumberFacts,
    Secondary,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("secondary", Secondary::reference())
}
pub fn definition() -> ApplicationQueryDefinition<NumericSchema, ReadQuery, Parameters, Row, Number>
{
    let shape = ApplicationQueryResultShapeBuilder::<
        NumericSchema,
        ReadQuery,
        Number,
        Row,
        ResultBinding,
    >::new(Number::reference())
    .field(primary_field())
    .field(secondary_field())
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
        .parameter(read_parameter())
        .where_equal(ReadEnabled::reference(), read_parameter())
        .build()
        .unwrap()
}
pub fn parameters() -> ApplicationQueryParameterSet<ReadQuery> {
    ApplicationQueryParameterSet::new()
        .bind(read_parameter(), true)
        .unwrap()
}
impl WorthQueryApplicationProjection<NumericSchema, ReadQuery> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, NumericSchema, ReadQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            primary: row.field(primary_field())?,
            secondary: row.field(secondary_field())?,
        })
    }
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Read {
    pub key: String,
}
worth_query_structured_value_binding!(pub ReadInputBinding for Read { identity: "numeric-correction.read-input.v1" });
pub struct ReadBinding;
type Scope = ApplicationQueryFieldScope<
    NumericSchema,
    Number,
    NumberFacts,
    Key,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationQueryBinding<NumericSchema> for ReadBinding {
    type Input = Read;
    type InputBinding = ReadInputBinding;
    type Query = ReadQuery;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = ActorBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "numeric-correction.read-binding.v1";
    // The unique key selects at most one numeric root.
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
    type Binding = ReadBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<ReadQuery> {
        parameters()
    }
    fn into_scope(self) -> Scope {
        Scope::new(Key::reference(), self.key)
    }
}
