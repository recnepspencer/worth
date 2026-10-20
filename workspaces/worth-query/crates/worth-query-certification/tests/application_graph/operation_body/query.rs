// Governed readback uses the same declared field bindings as ordinary queries.
use super::schema::*;
pub use crate::document_retention_model::retention_entry::DocumentRetentionRead;
pub use crate::document_retention_model::schema::{
    DocumentQueryParameters, DocumentQueryParametersBinding, DocumentRetentionRow,
    DocumentRetentionRowBinding,
};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::{
        WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
        WorthQueryApplicationProjectionRow,
    },
    worth_query_application_query, worth_query_portable_type,
};

pub struct IdentitySlot;
pub struct RetentionSlot;
worth_query_portable_type!(IdentitySlot => "worth.query.body-court.identity-slot.v1");
worth_query_portable_type!(RetentionSlot => "worth.query.body-court.retention-slot.v1");
worth_query_application_query!(pub DocumentRetentionQuery for Schema: BodySchemaBinding,
    identity "DocumentRetentionQuery", parameters DocumentQueryParametersBinding,
    result DocumentRetentionRowBinding, scope Document => "Document", name "body_retention_read");

fn identity_result<Schema: BodySchemaBinding>() -> ApplicationQueryResultFieldRef<
    DocumentRetentionQuery,
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
fn retention_result<Schema: BodySchemaBinding>() -> ApplicationQueryResultFieldRef<
    DocumentRetentionQuery,
    RetentionSlot,
    Schema,
    Document,
    DocumentFacts,
    DocumentRetentionField,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("retention", DocumentRetentionField::reference())
}
pub fn document_retention_query_definition<const DESTINATION: u64>() -> ApplicationQueryDefinition<
    DocumentRetentionSchema<DESTINATION>,
    DocumentRetentionQuery,
    DocumentQueryParameters,
    DocumentRetentionRow,
    Document,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        DocumentRetentionSchema<DESTINATION>,
        DocumentRetentionQuery,
        Document,
        DocumentRetentionRow,
        DocumentRetentionRowBinding,
    >::new(Document::reference())
    .field(identity_result())
    .field(retention_result())
    .build();
    ApplicationQueryDefinitionBuilder::declare(DocumentRetentionQuery::reference())
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
    WorthQueryApplicationProjection<DocumentRetentionSchema<DESTINATION>, DocumentRetentionQuery>
    for DocumentRetentionRow
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<
            '_,
            DocumentRetentionSchema<DESTINATION>,
            DocumentRetentionQuery,
        >,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(identity_result())?,
            retention_days: row.field(retention_result())?,
        })
    }
}
pub struct DocumentRetentionQueryBinding<const DESTINATION: u64>;
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
    for DocumentRetentionQueryBinding<DESTINATION>
{
    type Input = DocumentRetentionRead;
    type InputBinding =
        crate::document_retention_model::retention_entry::DocumentRetentionReadBinding;
    type Query = DocumentRetentionQuery;
    type ParameterBinding = DocumentQueryParametersBinding;
    type ResultBinding = DocumentRetentionRowBinding;
    type ScopeBinding = Scope<DESTINATION>;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.body-court.read-binding.v1";
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
    for DocumentRetentionRead
{
    type Binding = DocumentRetentionQueryBinding<DESTINATION>;
    fn parameters(&self) -> ApplicationQueryParameterSet<DocumentRetentionQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope<DESTINATION> {
        Scope::new(DocumentIdentityField::reference(), self.identity)
    }
}
