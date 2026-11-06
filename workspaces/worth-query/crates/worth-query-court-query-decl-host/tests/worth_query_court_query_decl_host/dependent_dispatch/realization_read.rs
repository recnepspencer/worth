//! Observe actual realization content independently of the producer's decision.
use super::split_model::*;
use super::*;
#[derive(Clone, Debug)]
pub struct ReadRealization(pub String);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Realized {
    pub key: String,
    pub value: u64,
}
worth_query_structured_value_binding!(pub Input for ReadRealization { identity: "dispatch.realization-read.input" });
worth_query_structured_value_binding!(pub ResultBinding for Realized { identity: "dispatch.realization-read.result" });
worth_query_application_query!(pub RealizationQuery for Schema, identity "RealizationQuery", parameters Parameters, result ResultBinding, scope Realization => "Realization", name "realization_read");
pub struct KeySlot;
pub struct ValueSlot;
worth_query_portable_type!(KeySlot => "dispatch.realization.key");
worth_query_portable_type!(ValueSlot => "dispatch.realization.value");
pub fn key_field(
) -> ApplicationQueryResultFieldRef<RealizationQuery, KeySlot, Schema, RealizationKey> {
    ApplicationQueryResultFieldRef::new("key", RealizationKey::at("RealizationKey"))
}
pub fn value_field(
) -> ApplicationQueryResultFieldRef<RealizationQuery, ValueSlot, Schema, RealizedValue> {
    ApplicationQueryResultFieldRef::new("value", RealizedValue::at("RealizedValue"))
}
pub fn definition(
) -> ApplicationQueryDefinition<Schema, RealizationQuery, (), Realized, Realization> {
    ApplicationQueryDefinitionBuilder::declare(RealizationQuery::reference())
        .root(Realization::reference())
        .scope(Realization::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                Schema,
                RealizationQuery,
                Realization,
                Realized,
                ResultBinding,
            >::new(Realization::reference())
            .field(key_field())
            .field(value_field())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<Schema, RealizationQuery> for Realized {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, RealizationQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key_field())?,
            value: row.field(value_field())?,
        })
    }
}
pub fn scope(input: ReadRealization) -> String {
    input.0
}
worth_query_query_binding!(pub Binding for ReadRealization, schema Schema, identity "dispatch.realization.read", input Input,
    query RealizationQuery, parameters Parameters => |_| ApplicationQueryParameterSet::new(), result ResultBinding,
    principal PrincipalBinding, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope RealizationKey, field RealizationKey::at("RealizationKey"), value scope, limits results 1);
