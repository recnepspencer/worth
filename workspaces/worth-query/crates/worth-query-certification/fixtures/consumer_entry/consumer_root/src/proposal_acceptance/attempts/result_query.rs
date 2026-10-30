//! Complete governed results collections admit disclosure before application key filtering.
use super::super::{
    model_query::ParametersBinding,
    schema::{data::Coordinates, *},
};
use super::{data::*, schema::*};
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResultRow {
    pub key: String,
    pub occurrence: Coordinates,
    pub data: StoredResult,
}
worth_query_structured_value_binding!(pub ResultBinding for ResultRow {identity:"proposal-application.results-row.v1"});
worth_query_application_query!(pub ResultQuery for Schema,identity "proposal-application.results-query.v1",
    parameters ParametersBinding,result ResultBinding,scope Model=>"Model",name "stored_results");
pub struct KeySlot;
pub struct DataSlot;
worth_query_portable_type!(KeySlot=>"proposal-application.results-key-slot.v1");
worth_query_portable_type!(DataSlot=>"proposal-application.results-data-slot.v1");
fn key() -> ApplicationQueryResultFieldRef<
    ResultQuery,
    KeySlot,
    Schema,
    AcceptanceResult,
    ResultFacts,
    ResultName,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("key", ResultName::reference())
}
fn data() -> ApplicationQueryResultFieldRef<
    ResultQuery,
    DataSlot,
    Schema,
    AcceptanceResult,
    ResultFacts,
    ResultData,
    StoredResult,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("data", ResultData::reference())
}
impl WorthQueryApplicationProjection<Schema, ResultQuery> for ResultRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, ResultQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let id = row.entity_id();
        Ok(Self {
            key: row.field(key())?,
            data: row.field(data())?,
            occurrence: Coordinates {
                partition: id.partition_value(),
                slot: id.local_slot_value(),
                generation: id.generation_value(),
            },
        })
    }
}
#[derive(Clone, Debug, WorthValue)]
pub struct ReadResults {
    pub model: String,
}
worth_query_structured_value_binding!(pub InputBinding for ReadResults {identity:"proposal-application.results-read.v1"});
fn scope(input: ReadResults) -> String {
    input.model
}
worth_query_query_binding!(pub Binding for ReadResults,schema Schema,
    identity "proposal-application.results-query-binding.v1",input InputBinding,
    query ResultQuery,parameters ParametersBinding=>|_| ApplicationQueryParameterSet::new(),result ResultBinding,
    principal PrincipalBinding,mapping ExternalMapping,principal_entity Principal,
    principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Model,ModelFacts,ModelName,String,ReadOnly,NoApplicationUnit,
    field ModelName::reference(),value scope,limits results 32);
pub fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    use super::super::access;
    let influence = ApplicationQueryInfluenceContract::permit_all();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "stored-results",
        access::ReadDomain::reference(),
    )
    .use_field_by(
        ModelName::reference(),
        access::encoded(1),
        influence.clone(),
    )
    .use_field_by(
        ResultName::reference(),
        access::encoded(1),
        influence.clone(),
    )
    .disclose_field_by(key(), access::encoded(1), influence.clone())
    .disclose_field_by(data(), access::encoded(1), influence);
    let definition = ApplicationQueryDefinitionBuilder::declare(ResultQuery::reference())
        .root(AcceptanceResult::reference())
        .scope(Model::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                Schema,
                ResultQuery,
                AcceptanceResult,
                ResultRow,
                ResultBinding,
            >::new(AcceptanceResult::reference())
            .field(key())
            .field(data())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(access::ReadResults::reference())
        .root_path(
            ApplicationQueryRootPath::from(Model::reference()).forward(ModelResult::reference()),
        )
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
        .build()
        .unwrap();
    builder
        .application_query(definition)
        .application_query_binding::<Binding>()
}
