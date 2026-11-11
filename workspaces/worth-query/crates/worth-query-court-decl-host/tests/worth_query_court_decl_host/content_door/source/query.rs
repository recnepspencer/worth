use super::*;
pub struct Parameters;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub key: String,
    pub content: ContentRefId,
}
worth_query_structured_value_binding!(pub ParametersBinding for Parameters {identity:"content-source.parameters.v1"});
worth_query_structured_value_binding!(pub RowBinding for Row {identity:"content-source.row.v1"});
worth_query_application_query!(pub SourceQuery for Schema,identity "content-source.source.v1",parameters ParametersBinding,result RowBinding,scope Record=>"Record",name "content_source");
worth_query_application_query!(pub ExactQuery for Schema,identity "content-source.exact.v1",parameters ParametersBinding,result RowBinding,scope Record=>"Record",name "content_exact");
pub struct KeySlot;
pub struct ContentSlot;
pub struct ProbeSlot;
worth_query_portable_type!(KeySlot=>"content-source.key-slot.v1");
worth_query_portable_type!(ContentSlot=>"content-source.content-slot.v1");
pub fn key<Q>() -> ApplicationQueryResultFieldRef<Q, KeySlot, Schema, Key> {
    ApplicationQueryResultFieldRef::new("key", Key::at("Key"))
}
pub fn content<Q>() -> ApplicationQueryResultFieldRef<Q, ContentSlot, Schema, Values> {
    ApplicationQueryResultFieldRef::new("samples", Values::at("Values"))
}
fn base<Q: ApplicationQueryMarkerIdentity<Schema, ResultBinding = RowBinding>>(
    reference: ApplicationQueryReference<Schema, Q, Parameters, Row, Record>,
) -> ApplicationQueryDefinitionBuilder<Schema, Q, Parameters, Row, Record> {
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Record::reference())
        .scope(Record::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Schema, Q, Record, Row, RowBinding>::new(
                Record::reference(),
            )
            .field(key())
            .field(content())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}
pub fn source() -> ApplicationQueryDefinitionBuilder<Schema, SourceQuery, Parameters, Row, Record> {
    base(SourceQuery::reference())
        .root_path(
            ApplicationQueryRootPath::from(Record::reference())
                .forward(ContainsSamples::reference()),
        )
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}
fn probe() -> ApplicationQueryParameterRef<ExactQuery, ProbeSlot, ProbeBinding> {
    ApplicationQueryParameterRef::from_query_identifier("minimum")
}
pub fn exact() -> ApplicationQueryDefinitionBuilder<Schema, ExactQuery, Parameters, Row, Record> {
    base(ExactQuery::reference())
        .managed_index_refined(Index::reference(), probe())
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}
impl<Q: ApplicationQueryMarkerIdentity<Schema>> WorthQueryApplicationProjection<Schema, Q> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Q>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key())?,
            content: row.field(content())?.0,
        })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct ReadSource {
    pub root: String,
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Find {
    pub root: String,
    pub minimum: u64,
}
worth_query_structured_value_binding!(pub SourceInput for ReadSource {identity:"content-source.source-input.v1"});
worth_query_structured_value_binding!(pub FindInput for Find {identity:"content-source.find-input.v1"});
fn source_root(input: ReadSource) -> String {
    input.root
}
fn find_root(input: Find) -> String {
    input.root
}
worth_query_query_binding!(pub SourceBinding for ReadSource,schema Schema,identity "content-source.source-binding.v1",input SourceInput,query SourceQuery,parameters ParametersBinding=>|_|ApplicationQueryParameterSet::new(),result RowBinding,
 principal Actor,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
 scope Key, field Key::at("Key"),value source_root,limits results 32);
worth_query_query_binding!(pub FindBinding for Find,schema Schema,identity "content-source.find-binding.v1",input FindInput,query ExactQuery,parameters ParametersBinding=>|input:&Find|ApplicationQueryParameterSet::new().bind(probe(),input.minimum).unwrap(),result RowBinding,
 principal Actor,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
 scope Key, field Key::at("Key"),value find_root,limits results 32);

worth_query_application_query!(pub StateQuery for Schema,identity "content-source.state.v1",parameters ParametersBinding,result RowBinding,scope Record=>"Record",name "content_state");
pub fn state() -> ApplicationQueryDefinitionBuilder<Schema, StateQuery, Parameters, Row, Record> {
    base(StateQuery::reference())
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct ReadState {
    pub root: String,
}
worth_query_structured_value_binding!(pub StateInput for ReadState {identity:"content-source.state-input.v1"});
fn state_root(input: ReadState) -> String {
    input.root
}
worth_query_query_binding!(pub StateBinding for ReadState,schema Schema,identity "content-source.state-binding.v1",input StateInput,query StateQuery,parameters ParametersBinding=>|_|ApplicationQueryParameterSet::new(),result RowBinding,
 principal Actor,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
 scope Key, field Key::at("Key"),value state_root,limits results 1);
