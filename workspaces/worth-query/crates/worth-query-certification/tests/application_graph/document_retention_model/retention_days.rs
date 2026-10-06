//! A second typed read of the same document: its retention in days as an
//! unsigned integer. With the Bool condition read, it gives expression
//! conditions two operands of different types over one source.

use worth_query_host::facade::declaration::{
    self,
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryBinding, ApplicationQueryBindingLimits,
        ApplicationQueryCardinality, ApplicationQueryDefinition, ApplicationQueryDefinitionBuilder,
        ApplicationQueryDependencyCeiling, ApplicationQueryDisclosureContract,
        ApplicationQueryFieldScope, ApplicationQueryIntent, ApplicationQueryLaneEligibility,
        ApplicationQueryParameterSet, ApplicationQueryResultFieldRef,
        ApplicationQueryResultShapeBuilder,
    },
    application_schema::{
        ApplicationFieldRef, ApplicationPrincipalBindingRef, ApplicationSchemaDeclarationBuilder,
        EqualityPredicate, NoApplicationUnit, ReadOnly, U64ApplicationValueBinding,
    },
};
use worth_query_host::facade::{
    primary_graph, worth_query_application_query, worth_query_portable_type,
    worth_query_structured_value_binding,
};

use super::schema::{
    Document, DocumentFacts, DocumentIdentityField, DocumentPrincipalBinding,
    DocumentQueryParametersBinding, DocumentRetentionField, DocumentRetentionSchema,
    ExternalMapping, Principal,
};

pub struct DocumentRetentionDaysSlot;
worth_query_portable_type!(DocumentRetentionDaysSlot => "worth.query.certification.document-retention.days-slot.v1");

worth_query_structured_value_binding!(pub DocumentRetentionDaysBinding for u64 {
    identity: "worth.query.certification.document-retention.days.v1"
});
worth_query_application_query!(
    pub DocumentRetentionDaysQuery for DocumentRetentionSchema,
    identity "DocumentRetentionDaysQuery",
    parameters DocumentQueryParametersBinding,
    result DocumentRetentionDaysBinding,
    scope Document => "Document",
    name "document_retention_days_query"
);

pub fn document_retention_days_query_definition() -> ApplicationQueryDefinition<
    DocumentRetentionSchema,
    DocumentRetentionDaysQuery,
    super::schema::DocumentQueryParameters,
    u64,
    Document,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        DocumentRetentionSchema,
        DocumentRetentionDaysQuery,
        Document,
        u64,
        DocumentRetentionDaysBinding,
    >::new(Document::reference())
    .field(days_result())
    .build();
    ApplicationQueryDefinitionBuilder::declare(DocumentRetentionDaysQuery::reference())
        .root(Document::reference())
        .scope(Document::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 3))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .expect("the document days query is canonical")
}

fn days_result() -> ApplicationQueryResultFieldRef<
    DocumentRetentionDaysQuery,
    DocumentRetentionDaysSlot,
    DocumentRetentionSchema,
    Document,
    DocumentFacts,
    DocumentRetentionField,
    u64,
    declaration::application_schema::ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("retention", DocumentRetentionField::reference())
}

impl
    primary_graph::WorthQueryApplicationProjection<
        DocumentRetentionSchema,
        DocumentRetentionDaysQuery,
    > for u64
{
    fn project(
        row: &primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            DocumentRetentionSchema,
            DocumentRetentionDaysQuery,
        >,
    ) -> Result<Self, primary_graph::WorthQueryApplicationProjectionDenial> {
        row.field(days_result())
    }
}

pub struct DocumentRetentionDaysQueryBinding;

type DocumentRetentionDaysScope = ApplicationQueryFieldScope<
    DocumentRetentionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl ApplicationQueryBinding<DocumentRetentionSchema> for DocumentRetentionDaysQueryBinding {
    type Input = DocumentRetentionDaysRead;
    type InputBinding = DocumentRetentionDaysReadBinding;
    type Query = DocumentRetentionDaysQuery;
    type ParameterBinding = DocumentQueryParametersBinding;
    type ResultBinding = DocumentRetentionDaysBinding;
    type ScopeBinding = DocumentRetentionDaysScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;

    const IDENTITY: &'static str =
        "worth.query.certification.document-retention.days-read-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::bounded(1, 64);

    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema,
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
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}

/// The request for one document's retention in days.
#[derive(Clone, Debug)]
pub struct DocumentRetentionDaysRead {
    pub identity: String,
}

worth_query_structured_value_binding!(pub DocumentRetentionDaysReadBinding for DocumentRetentionDaysRead {
    identity: "worth.query.certification.document-retention.days-read.v1"
});

impl ApplicationQueryIntent<DocumentRetentionSchema> for DocumentRetentionDaysRead {
    type Binding = DocumentRetentionDaysQueryBinding;

    fn parameters(&self) -> ApplicationQueryParameterSet<DocumentRetentionDaysQuery> {
        ApplicationQueryParameterSet::new()
    }

    fn into_scope(self) -> DocumentRetentionDaysScope {
        DocumentRetentionDaysScope::new(DocumentIdentityField::reference(), self.identity)
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema.application_query_binding::<DocumentRetentionDaysQueryBinding>()
}
