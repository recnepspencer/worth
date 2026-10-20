//! Governed readback distinguishes missing draft note from present empty text.
use super::query::{DocumentQueryParameters, DocumentQueryParametersBinding};
use super::schema::*;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestExt,
    declaration::{application_query::*, application_schema::*},
    primary_graph::{
        WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
        WorthQueryApplicationProjectionRow,
    },
    worth_query_application_query, worth_query_portable_type, worth_query_structured_value_binding,
};

#[derive(Clone, Debug)]
pub struct DraftRead {
    pub identity: String,
}
#[derive(Clone, Debug)]
pub struct DraftRow {
    pub identity: String,
    pub note: Option<String>,
    pub audit_source: u64,
}
worth_query_portable_type!(DraftRow => "worth.query.body-court.draft-row.v1");
worth_query_structured_value_binding!(pub DraftReadBinding for DraftRead {
    identity: "worth.query.body-court.draft-read.v1"
});
worth_query_structured_value_binding!(pub DraftRowBinding for DraftRow {
    identity: "worth.query.body-court.draft-row.v1"
});
pub struct IdentitySlot;
pub struct NoteSlot;
pub struct AuditSourceSlot;
worth_query_portable_type!(IdentitySlot => "worth.query.body-court.draft-identity-slot.v1");
worth_query_portable_type!(NoteSlot => "worth.query.body-court.draft-note-slot.v1");
worth_query_portable_type!(AuditSourceSlot => "worth.query.body-court.draft-audit-source-slot.v1");
worth_query_application_query!(pub DraftQuery for Schema: BodySchemaBinding,
    identity "DraftQuery", parameters DocumentQueryParametersBinding,
    result DraftRowBinding, scope Document => "Document", name "body_draft_read");

fn identity<Schema: BodySchemaBinding>() -> ApplicationQueryResultFieldRef<
    DraftQuery,
    IdentitySlot,
    Schema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("identity", DocumentIdentityField::reference())
}
fn note<Schema: BodySchemaBinding>() -> ApplicationQueryOptionalResultFieldRef<
    DraftQuery,
    NoteSlot,
    Schema,
    Document,
    DocumentDraftFacts,
    DocumentDraftNoteField,
    String,
    ReadWrite,
    NoEqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryOptionalResultFieldRef::new("note", DocumentDraftNoteField::reference())
}
fn audit_source<Schema: BodySchemaBinding>() -> ApplicationQueryResultFieldRef<
    DraftQuery,
    AuditSourceSlot,
    Schema,
    Document,
    DocumentFacts,
    DocumentAuditSourceField,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("audit_source", DocumentAuditSourceField::reference())
}

pub fn draft_query_definition<const DESTINATION: u64>() -> ApplicationQueryDefinition<
    DocumentRetentionSchema<DESTINATION>,
    DraftQuery,
    DocumentQueryParameters,
    DraftRow,
    Document,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        DocumentRetentionSchema<DESTINATION>,
        DraftQuery,
        Document,
        DraftRow,
        DraftRowBinding,
    >::new(Document::reference())
    .field(identity())
    .optional_field(note())
    .field(audit_source())
    .build();
    ApplicationQueryDefinitionBuilder::declare(DraftQuery::reference())
        .root(Document::reference())
        .scope(Document::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
impl<const DESTINATION: u64>
    WorthQueryApplicationProjection<DocumentRetentionSchema<DESTINATION>, DraftQuery> for DraftRow
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<
            '_,
            DocumentRetentionSchema<DESTINATION>,
            DraftQuery,
        >,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(identity())?,
            note: row.optional_field(note())?,
            audit_source: row.field(audit_source())?,
        })
    }
}
pub struct DraftQueryBinding<const DESTINATION: u64>;
type Scope<const DESTINATION: u64> = ApplicationQueryFieldScope<
    DocumentRetentionSchema<DESTINATION>,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl<const DESTINATION: u64> ApplicationQueryBinding<DocumentRetentionSchema<DESTINATION>>
    for DraftQueryBinding<DESTINATION>
{
    type Input = DraftRead;
    type InputBinding = DraftReadBinding;
    type Query = DraftQuery;
    type ParameterBinding = DocumentQueryParametersBinding;
    type ResultBinding = DraftRowBinding;
    type ScopeBinding = Scope<DESTINATION>;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.body-court.draft-query-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema<DESTINATION>,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        DocumentIdentityField::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema<DESTINATION>,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
impl<const DESTINATION: u64> ApplicationQueryIntent<DocumentRetentionSchema<DESTINATION>>
    for DraftRead
{
    type Binding = DraftQueryBinding<DESTINATION>;
    fn parameters(&self) -> ApplicationQueryParameterSet<DraftQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope<DESTINATION> {
        Scope::new(DocumentIdentityField::reference(), self.identity)
    }
}

pub fn read_note<const DESTINATION: u64>(
    app: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        super::program::BodyProgram<DESTINATION>,
    >,
) -> Option<String> {
    read_row(app).note
}

pub fn read_audit_source<const DESTINATION: u64>(
    app: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        super::program::BodyProgram<DESTINATION>,
    >,
) -> u64 {
    read_row(app).audit_source
}

fn read_row<const DESTINATION: u64>(
    app: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        super::program::BodyProgram<DESTINATION>,
    >,
) -> DraftRow {
    let scope = super::request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    let result = app
        .request(&principal, &scope)
        .query(DraftRead {
            identity: super::DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap();
    let row = &result.rows()[0];
    assert_eq!(row.identity, super::DOCUMENT_IDENTITY);
    row.clone()
}
