//! Proposal loading consumes an ordinary declared canonical field decoder.
use super::{
    model_query::ParametersBinding,
    schema::{
        data::{Coordinates, StoredProposal},
        *,
    },
};
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalRow {
    pub key: String,
    pub occurrence: Coordinates,
    pub data: StoredProposal,
}
worth_query_structured_value_binding!(pub ResultBinding for ProposalRow {identity:"proposal-application.proposal-row.v1"});
worth_query_application_query!(pub ProposalQuery for Schema, identity "proposal-application.proposal-query.v1",
    parameters ParametersBinding,result ResultBinding,scope Model=>"Model",name "stored_proposals");
pub struct KeySlot;
pub struct DataSlot;
worth_query_portable_type!(KeySlot => "proposal-application.proposal-key-slot.v1");
worth_query_portable_type!(DataSlot => "proposal-application.proposal-data-slot.v1");
fn key() -> ApplicationQueryResultFieldRef<
    ProposalQuery,
    KeySlot,
    Schema,
    Proposal,
    ProposalFacts,
    ProposalName,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("key", ProposalName::reference())
}
fn data() -> ApplicationQueryResultFieldRef<
    ProposalQuery,
    DataSlot,
    Schema,
    Proposal,
    ProposalFacts,
    ProposalData,
    StoredProposal,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("data", ProposalData::reference())
}
impl WorthQueryApplicationProjection<Schema, ProposalQuery> for ProposalRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, ProposalQuery>,
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
pub struct ReadProposals {
    pub model: String,
}
worth_query_structured_value_binding!(pub InputBinding for ReadProposals {identity:"proposal-application.proposals-read.v1"});
fn scope(input: ReadProposals) -> String {
    input.model
}
worth_query_query_binding!(pub Binding for ReadProposals,schema Schema,
    identity "proposal-application.proposal-query-binding.v1",input InputBinding,
    query ProposalQuery,parameters ParametersBinding=>|_| ApplicationQueryParameterSet::new(),result ResultBinding,
    principal PrincipalBinding,mapping ExternalMapping,principal_entity Principal,
    principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Model,ModelFacts,ModelName,String,ReadOnly,NoApplicationUnit,
    field ModelName::reference(),value scope,limits results 32);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    use super::access;
    let influence = ApplicationQueryInfluenceContract::permit_all();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "stored-proposals",
        access::ReadDomain::reference(),
    )
    .use_field_by(
        ModelName::reference(),
        access::encoded(1),
        influence.clone(),
    )
    .use_field_by(
        ProposalName::reference(),
        access::encoded(1),
        influence.clone(),
    )
    .disclose_field_by(key(), access::encoded(1), influence.clone())
    .disclose_field_by(data(), access::encoded(1), influence);
    let definition = ApplicationQueryDefinitionBuilder::declare(ProposalQuery::reference())
        .root(Proposal::reference())
        .scope(Model::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                Schema,
                ProposalQuery,
                Proposal,
                ProposalRow,
                ResultBinding,
            >::new(Proposal::reference())
            .field(key())
            .field(data())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(access::ReadProposals::reference())
        .root_path(
            ApplicationQueryRootPath::from(Model::reference()).forward(ModelProposal::reference()),
        )
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
        .build()
        .unwrap();
    schema
        .application_query(definition)
        .application_query_binding::<Binding>()
}
