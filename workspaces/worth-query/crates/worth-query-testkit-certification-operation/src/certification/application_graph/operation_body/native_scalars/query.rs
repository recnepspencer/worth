//! Governed readback distinguishes absent native carriers from present ones.
pub use super::super::query::{DocumentQueryParameters, DocumentQueryParametersBinding};
pub use super::super::schema::*;
use super::records::{
    DocumentDueDateField, DocumentOpensAtField, DocumentPayloadField, DocumentTokenField,
};
use worth_foundational::facade::{CanonicalDate, CanonicalTime, InlineBytes};
pub use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestExt,
    declaration::{application_query::*, application_schema::*},
    primary_graph::{
        WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
        WorthQueryApplicationProjectionRow,
    },
    worth_query_application_query, worth_query_portable_type, worth_query_structured_value_binding,
};

#[derive(Clone, Debug)]
pub struct ScheduleRead {
    pub identity: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleRow {
    pub due: Option<CanonicalDate>,
    pub opens: Option<CanonicalTime>,
    pub token: Option<[u8; 16]>,
    pub payload: Option<InlineBytes>,
}
worth_query_portable_type!(ScheduleRow => "worth.query.body-court.schedule-row.v1");
worth_query_structured_value_binding!(pub ScheduleReadBinding for ScheduleRead {
    identity: "worth.query.body-court.schedule-read.v1"
});
worth_query_structured_value_binding!(pub ScheduleRowBinding for ScheduleRow {
    identity: "worth.query.body-court.schedule-row.v1"
});
pub struct DueSlot;
pub struct OpensSlot;
pub struct TokenSlot;
pub struct PayloadSlot;
worth_query_portable_type!(DueSlot => "worth.query.body-court.schedule-due-slot.v1");
worth_query_portable_type!(OpensSlot => "worth.query.body-court.schedule-opens-slot.v1");
worth_query_portable_type!(TokenSlot => "worth.query.body-court.schedule-token-slot.v1");
worth_query_portable_type!(PayloadSlot => "worth.query.body-court.schedule-payload-slot.v1");
worth_query_application_query!(pub ScheduleQuery for Schema: BodySchemaBinding,
    identity "ScheduleQuery", parameters DocumentQueryParametersBinding,
    result ScheduleRowBinding, scope Document => "Document", name "body_schedule_read");

type Optional<Schema, Slot, Field> =
    ApplicationQueryOptionalResultFieldRef<ScheduleQuery, Slot, Schema, Field>;
fn due<Schema: BodySchemaBinding>() -> Optional<Schema, DueSlot, DocumentDueDateField> {
    ApplicationQueryOptionalResultFieldRef::new(
        "due",
        DocumentDueDateField::at("DocumentDueDateField"),
    )
}
fn opens<Schema: BodySchemaBinding>() -> Optional<Schema, OpensSlot, DocumentOpensAtField> {
    ApplicationQueryOptionalResultFieldRef::new(
        "opens",
        DocumentOpensAtField::at("DocumentOpensAtField"),
    )
}
fn token<Schema: BodySchemaBinding>() -> Optional<Schema, TokenSlot, DocumentTokenField> {
    ApplicationQueryOptionalResultFieldRef::new(
        "token",
        DocumentTokenField::at("DocumentTokenField"),
    )
}
fn payload<Schema: BodySchemaBinding>() -> Optional<Schema, PayloadSlot, DocumentPayloadField> {
    ApplicationQueryOptionalResultFieldRef::new(
        "payload",
        DocumentPayloadField::at("DocumentPayloadField"),
    )
}

pub fn schedule_query_definition<const MODE: u64>() -> ApplicationQueryDefinitionBuilder<
    DocumentRetentionSchema<MODE>,
    ScheduleQuery,
    DocumentQueryParameters,
    ScheduleRow,
    Document,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        DocumentRetentionSchema<MODE>,
        ScheduleQuery,
        Document,
        ScheduleRow,
        ScheduleRowBinding,
    >::new(Document::reference())
    .optional_field(due())
    .optional_field(opens())
    .optional_field(token())
    .optional_field(payload())
    .build();
    ApplicationQueryDefinitionBuilder::declare(ScheduleQuery::reference())
        .root(Document::reference())
        .scope(Document::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}

impl<const MODE: u64> WorthQueryApplicationProjection<DocumentRetentionSchema<MODE>, ScheduleQuery>
    for ScheduleRow
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, DocumentRetentionSchema<MODE>, ScheduleQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            due: row.optional_field(due())?,
            opens: row.optional_field(opens())?,
            token: row.optional_field(token())?,
            payload: row.optional_field(payload())?,
        })
    }
}

pub struct ScheduleQueryBinding<const MODE: u64>;
impl<const MODE: u64> ApplicationQueryBinding<DocumentRetentionSchema<MODE>>
    for ScheduleQueryBinding<MODE>
{
    type Input = ScheduleRead;
    type InputBinding = ScheduleReadBinding;
    type Query = ScheduleQuery;
    type ParameterBinding = DocumentQueryParametersBinding;
    type ResultBinding = ScheduleRowBinding;
    type ScopeBinding = ApplicationQueryFieldScope<DocumentIdentityField>;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.body-court.schedule-query-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> DocumentIdentityField {
        DocumentIdentityField::at("DocumentIdentityField")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema<MODE>,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
impl<const MODE: u64> ApplicationQueryIntent<DocumentRetentionSchema<MODE>> for ScheduleRead {
    type Binding = ScheduleQueryBinding<MODE>;
    fn parameters(&self) -> ApplicationQueryParameterSet<ScheduleQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> ApplicationQueryFieldScope<DocumentIdentityField> {
        ApplicationQueryFieldScope::new(
            DocumentIdentityField::at("DocumentIdentityField"),
            self.identity,
        )
    }
}

pub fn read_row(app: &super::Application) -> ScheduleRow {
    let scope = super::request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    let result = app
        .request(&principal, &scope)
        .query(ScheduleRead {
            identity: super::DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap();
    result.rows()[0].clone()
}
