//! Ordinary readback of the actual stored derived arithmetic field.
use super::{
    query::{Parameters, ParametersBinding},
    schema::*,
};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};
pub struct Slot;
worth_query_portable_type!(Slot => "numeric-correction.calculated-slot.v1");
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub value: u64,
}
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "numeric-correction.calculated-row.v1" });
worth_query_application_query!(pub Query for NumericSchema, identity "numeric-correction.calculation-query.v1",
    parameters ParametersBinding, result ResultBinding, scope Number => "Number", name "calculation");
fn field() -> ApplicationQueryResultFieldRef<
    Query,
    Slot,
    NumericSchema,
    Number,
    NumberFacts,
    Calculated,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("calculated", Calculated::reference())
}
pub fn definition() -> ApplicationQueryDefinition<NumericSchema, Query, Parameters, Row, Number> {
    let shape = ApplicationQueryResultShapeBuilder::<
        NumericSchema,
        Query,
        Number,
        Row,
        ResultBinding,
    >::new(Number::reference())
    .field(field())
    .build();
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
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
impl WorthQueryApplicationProjection<NumericSchema, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, NumericSchema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            value: row.field(field())?,
        })
    }
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Read {
    pub key: String,
}
worth_query_structured_value_binding!(pub InputBinding for Read { identity: "numeric-correction.calculated-read-input.v1" });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<
    NumericSchema,
    Number,
    NumberFacts,
    Key,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationQueryBinding<NumericSchema> for Binding {
    type Input = Read;
    type InputBinding = InputBinding;
    type Query = Query;
    type ParameterBinding = super::query::ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = ActorBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "numeric-correction.calculated-read.v1";
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
    fn parameters(&self) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope {
        Scope::new(Key::reference(), self.key)
    }
}
