//! Source and indexed query project the same required child of an optional parent.
use super::super::query::{Parameters, ParametersBinding};
use super::*;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub key: String,
    pub content: Option<ContentRefId>,
}
worth_query_structured_value_binding!(pub RowBinding for Row {identity:"optional-content.row.v1"});
worth_query_application_query!(pub OptionalSourceQuery for Schema,identity "optional-content.source.v1",parameters ParametersBinding,result RowBinding,scope Record=>"Record",name "optional_content_source");
worth_query_application_query!(pub OptionalFindQuery for Schema,identity "optional-content.find.v1",parameters ParametersBinding,result RowBinding,scope Record=>"Record",name "optional_content_find");
pub struct KeySlot;
pub struct ContentSlot;
pub struct ProbeSlot;
worth_query_portable_type!(KeySlot=>"optional-content.key-slot.v1");
worth_query_portable_type!(ContentSlot=>"optional-content.content-slot.v1");
pub fn key<Q>() -> ApplicationQueryResultFieldRef<Q, KeySlot, Schema, Key> {
    ApplicationQueryResultFieldRef::new("key", Key::at("Key"))
}
pub fn content<Q>() -> ApplicationQueryOptionalAspectFieldRef<Q, ContentSlot, Schema, Values> {
    ApplicationQueryOptionalAspectFieldRef::new("samples", Values::at("Values"))
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
            .optional_aspect_field(content())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}
pub fn source(
) -> ApplicationQueryDefinitionBuilder<Schema, OptionalSourceQuery, Parameters, Row, Record> {
    base(OptionalSourceQuery::reference())
        .root_path(
            ApplicationQueryRootPath::from(Record::reference())
                .forward(ContainsOptionalSamples::reference()),
        )
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}
fn probe() -> ApplicationQueryParameterRef<OptionalFindQuery, ProbeSlot, ProbeBinding> {
    ApplicationQueryParameterRef::from_query_identifier("probe")
}
pub fn find(
) -> ApplicationQueryDefinitionBuilder<Schema, OptionalFindQuery, Parameters, Row, Record> {
    base(OptionalFindQuery::reference())
        .managed_index(Index::reference(), probe())
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}
impl<Q: ApplicationQueryMarkerIdentity<Schema>> WorthQueryApplicationProjection<Schema, Q> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Q>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key())?,
            content: row.optional_aspect_field(content())?.map(|value| value.0),
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
}
worth_query_structured_value_binding!(pub SourceInput for ReadSource {identity:"optional-content.source-input.v1"});
worth_query_structured_value_binding!(pub FindInput for Find {identity:"optional-content.find-input.v1"});
fn source_root(input: ReadSource) -> String {
    input.root
}
fn find_root(input: Find) -> String {
    input.root
}
worth_query_query_binding!(pub SourceBinding for ReadSource,schema Schema,identity "optional-content.source-binding.v1",input SourceInput,query OptionalSourceQuery,parameters ParametersBinding=>|_|ApplicationQueryParameterSet::new(),result RowBinding,
 principal Actor,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
 scope Key,field Key::at("Key"),value source_root,limits results 32);
worth_query_query_binding!(pub FindBinding for Find,schema Schema,identity "optional-content.find-binding.v1",input FindInput,query OptionalFindQuery,parameters ParametersBinding=>|_|ApplicationQueryParameterSet::new().bind(probe(),0u64).unwrap(),result RowBinding,
 principal Actor,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
 scope Key,field Key::at("Key"),value find_root,limits results 32);
