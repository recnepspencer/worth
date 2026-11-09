//! Current numeric rows; a filtered-out subject cannot disclose its correction history.
pub mod full;
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
pub struct OperandSlot;
worth_query_portable_type!(OperandSlot => "numeric-correction.readquery-operand-slot.v1");
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub operand: Option<u64>,
    pub primary: u64,
    pub secondary: u64,
}
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "numeric-correction.parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "numeric-correction.row.v1" });
worth_query_application_query!(pub ReadQuery for NumericSchema, identity "numeric-correction.read.v1",
    parameters ParametersBinding, result ResultBinding, scope Number => "Number", name "number");
pub fn read_parameter(
) -> ApplicationQueryParameterRef<ReadQuery, ReadParameter, BoolApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("read_enabled")
}
pub fn primary_field(
) -> ApplicationQueryResultFieldRef<ReadQuery, PrimarySlot, NumericSchema, Primary> {
    ApplicationQueryResultFieldRef::new("primary", Primary::at("Primary"))
}
pub fn secondary_field(
) -> ApplicationQueryResultFieldRef<ReadQuery, SecondarySlot, NumericSchema, Secondary> {
    ApplicationQueryResultFieldRef::new("secondary", Secondary::at("Secondary"))
}
fn operand(
) -> ApplicationQueryOptionalResultFieldRef<ReadQuery, OperandSlot, NumericSchema, Operand> {
    ApplicationQueryOptionalResultFieldRef::new("operand", Operand::at("Operand"))
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
    .field(primary_field())
    .field(secondary_field())
    .optional_field(operand())
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
        .where_equal(ReadEnabled::at("ReadEnabled"), read_parameter())
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
            operand: row.optional_field(operand())?,
            primary: row.field(primary_field())?.0,
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
pub type Scope = ApplicationQueryFieldScope<Key>;
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
impl ApplicationQueryIntent<NumericSchema> for Read {
    type Binding = ReadBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<ReadQuery> {
        parameters()
    }
    fn into_scope(self) -> Scope {
        Scope::new(Key::at("Key"), self.key)
    }
}
