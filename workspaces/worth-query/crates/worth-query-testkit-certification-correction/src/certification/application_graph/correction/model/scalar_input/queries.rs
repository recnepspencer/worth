//! Separate ordinary producer data and governed source-disclosure audiences.
use super::super::{
    causal_inspection::capability,
    query::{Parameters, ParametersBinding},
    schema::*,
};
use super::{ScalarFacts, Value};
use worth_query_decl::facade::worth_query_query_binding;
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};

macro_rules! query {
    ($module:ident, $name:literal, $identity:literal, $result:literal, $input:literal, $binding:literal,
     $primary_slot:literal, $scalar_slot:literal, $audience:ident) => {
        pub mod $module {
            use super::*;
            pub struct PrimarySlot;
            pub struct ScalarSlot;
            worth_query_portable_type!(PrimarySlot => $primary_slot);
            worth_query_portable_type!(ScalarSlot => $scalar_slot);
            #[derive(Clone, Debug, Eq, PartialEq)]
            pub struct Row { pub input: u64, pub scalar: Option<u64> }
            worth_query_structured_value_binding!(pub ResultBinding for Row { identity: $result });
            worth_query_application_query!(pub Query for NumericSchema, identity $identity,
                parameters ParametersBinding, result ResultBinding, scope Number => "Number", name $name);
            fn primary() -> ApplicationQueryResultFieldRef<Query, PrimarySlot, NumericSchema, Number, NumberFacts, Primary, u64, ReadWrite, EqualityPredicate, NoApplicationUnit> {
                ApplicationQueryResultFieldRef::new("input", Primary::reference())
            }
            fn scalar() -> ApplicationQueryOptionalAspectFieldRef<Query, ScalarSlot, NumericSchema, Number, ScalarFacts, Value, u64, ReadWrite, EqualityPredicate, NoApplicationUnit> {
                ApplicationQueryOptionalAspectFieldRef::new("scalar", Value::reference())
            }
            pub fn definition() -> ApplicationQueryDefinition<NumericSchema, Query, Parameters, Row, Number> {
                let shape = ApplicationQueryResultShapeBuilder::<NumericSchema, Query, Number, Row, ResultBinding>::new(Number::reference())
                    .field(primary()).optional_aspect_field(scalar()).build();
                ApplicationQueryDefinitionBuilder::declare(Query::reference())
                    .root(Number::reference()).scope(Number::reference()).result_shape(shape)
                    .cardinality(ApplicationQueryCardinality::Many).disclosure(query!(@disclosure $audience, primary(), scalar()))
                    .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
                    .lanes(ApplicationQueryLaneEligibility::one_shot()).public().build().unwrap()
            }
            impl WorthQueryApplicationProjection<NumericSchema, Query> for Row {
                fn project(row: &WorthQueryApplicationProjectionRow<'_, NumericSchema, Query>) -> Result<Self, WorthQueryApplicationProjectionDenial> {
                    Ok(Self { input: row.field(primary())?, scalar: row.optional_aspect_field(scalar())? })
                }
            }
            #[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
            pub struct Read { pub key: String }
            impl Read { fn into_key(self) -> String { self.key } }
            worth_query_structured_value_binding!(pub InputBinding for Read { identity: $input });
            worth_query_query_binding!(pub Binding for Read, schema NumericSchema, identity $binding,
                input InputBinding, query Query, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(), result ResultBinding,
                principal ActorBinding, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
                scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
                field Key::reference(), value Read::into_key, limits results 1);
        }
    };
    (@disclosure ordinary, $primary:expr, $scalar:expr) => { ApplicationQueryDisclosureContract::public() };
    (@disclosure governed, $primary:expr, $scalar:expr) => {
        ApplicationQueryDisclosureContract::governed_by("numeric-causal-diagnostic", capability::Inspect::reference())
            .disclose_field_by($primary, capability::text("calculation"), ApplicationQueryInfluenceContract::permit_all())
            .disclose_optional_aspect_field_by($scalar, capability::text("calculation"), ApplicationQueryInfluenceContract::permit_all())
    };
}
query!(
    data,
    "scalar-producer-input",
    "numeric-scalar.data.v1",
    "numeric-scalar.data-row.v1",
    "numeric-scalar.data-input.v1",
    "numeric-scalar.data-binding.v1",
    "numeric-scalar.data-primary.v1",
    "numeric-scalar.data-value.v1",
    ordinary
);
query!(
    inspection,
    "scalar-input-inspection",
    "numeric-scalar.inspection.v1",
    "numeric-scalar.inspection-row.v1",
    "numeric-scalar.inspection-input.v1",
    "numeric-scalar.inspection-binding.v1",
    "numeric-scalar.inspection-primary.v1",
    "numeric-scalar.inspection-value.v1",
    governed
);
