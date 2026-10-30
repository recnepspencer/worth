//! Complete governed claims collections admit disclosure before application key filtering.
use super::super::{
    model_query::ParametersBinding,
    schema::{data::Coordinates, *},
};
use super::{data::*, schema::*};
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimRow {
    pub key: String,
    pub occurrence: Coordinates,
    pub data: StoredClaim,
}
worth_query_structured_value_binding!(pub ResultBinding for ClaimRow {identity:"proposal-application.claims-row.v1"});
worth_query_application_query!(pub ClaimQuery for Schema,identity "proposal-application.claims-query.v1",
    parameters ParametersBinding,result ResultBinding,scope Model=>"Model",name "stored_claims");
pub struct KeySlot;
pub struct DataSlot;
worth_query_portable_type!(KeySlot=>"proposal-application.claims-key-slot.v1");
worth_query_portable_type!(DataSlot=>"proposal-application.claims-data-slot.v1");
fn key() -> ApplicationQueryResultFieldRef<
    ClaimQuery,
    KeySlot,
    Schema,
    AcceptanceClaim,
    ClaimFacts,
    ClaimName,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("key", ClaimName::reference())
}
fn data() -> ApplicationQueryResultFieldRef<
    ClaimQuery,
    DataSlot,
    Schema,
    AcceptanceClaim,
    ClaimFacts,
    ClaimData,
    StoredClaim,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("data", ClaimData::reference())
}
impl WorthQueryApplicationProjection<Schema, ClaimQuery> for ClaimRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, ClaimQuery>,
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
pub struct ReadClaims {
    pub model: String,
}
worth_query_structured_value_binding!(pub InputBinding for ReadClaims {identity:"proposal-application.claims-read.v1"});
fn scope(input: ReadClaims) -> String {
    input.model
}
worth_query_query_binding!(pub Binding for ReadClaims,schema Schema,
    identity "proposal-application.claims-query-binding.v1",input InputBinding,
    query ClaimQuery,parameters ParametersBinding=>|_| ApplicationQueryParameterSet::new(),result ResultBinding,
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
        "stored-claims",
        access::ReadDomain::reference(),
    )
    .use_field_by(
        ModelName::reference(),
        access::encoded(1),
        influence.clone(),
    )
    .use_field_by(
        ClaimName::reference(),
        access::encoded(1),
        influence.clone(),
    )
    .disclose_field_by(key(), access::encoded(1), influence.clone())
    .disclose_field_by(data(), access::encoded(1), influence);
    let definition = ApplicationQueryDefinitionBuilder::declare(ClaimQuery::reference())
        .root(AcceptanceClaim::reference())
        .scope(Model::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                Schema,
                ClaimQuery,
                AcceptanceClaim,
                ClaimRow,
                ResultBinding,
            >::new(AcceptanceClaim::reference())
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
            ApplicationQueryRootPath::from(Model::reference()).forward(ModelClaim::reference()),
        )
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
        .build()
        .unwrap();
    builder
        .application_query(definition)
        .application_query_binding::<Binding>()
}
