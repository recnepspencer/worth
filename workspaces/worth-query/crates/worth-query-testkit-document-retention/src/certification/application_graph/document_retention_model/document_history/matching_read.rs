//! A real scoped READ only returns a document while its retention matches the authored filter.
use super::super::record_history::*;
use super::super::retention_entry::DocumentRetentionQueryScope;
use worth_query_decl::facade::{
    worth_query_application_query, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

#[derive(Clone, Debug)]
pub struct Input {
    pub identity: String,
}
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "worth.query.document-history.matching-read-input.v1" });
pub struct Parameters {
    pub expected: u64,
}
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "worth.query.document-history.matching-read-parameters.v1" });
worth_query_application_query!(pub Query for DocumentRetentionSchema, identity "HistoryMatchingRetentionRead", parameters ParametersBinding,
    result DocumentRetentionConditionBinding, scope Document => "Document", name "history_matching_retention_read");
pub struct Expected;
pub struct Slot;
worth_query_portable_type!(Slot => "worth.query.document-history.matching-retention-slot.v1");
fn expected() -> ApplicationQueryParameterRef<Query, Expected, U64ApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("expected")
}
fn retention() -> ApplicationQueryResultFieldRef<
    Query,
    Slot,
    DocumentRetentionSchema,
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
pub struct Binding;
impl ApplicationQueryBinding<DocumentRetentionSchema> for Binding {
    type Input = Input;
    type InputBinding = InputBinding;
    type Query = Query;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = DocumentRetentionConditionBinding;
    type ScopeBinding = DocumentRetentionQueryScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.document-history.matching-read-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
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
impl ApplicationQueryIntent<DocumentRetentionSchema> for Input {
    type Binding = Binding;
    fn parameters(&self) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
            .bind(expected(), 7)
            .unwrap()
    }
    fn into_scope(self) -> DocumentRetentionQueryScope {
        DocumentRetentionQueryScope::new(DocumentIdentityField::reference(), self.identity)
    }
}
pub fn definition(
) -> ApplicationQueryDefinition<DocumentRetentionSchema, Query, Parameters, bool, Document> {
    let shape = ApplicationQueryResultShapeBuilder::<
        DocumentRetentionSchema,
        Query,
        Document,
        bool,
        DocumentRetentionConditionBinding,
    >::new(Document::reference())
    .field(retention())
    .build();
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Document::reference())
        .scope(Document::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(expected())
        .where_equal(DocumentRetentionField::reference(), expected())
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<DocumentRetentionSchema, Query> for bool {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, DocumentRetentionSchema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(row.field(retention())? == 7)
    }
}
pub struct History;
worth_query_portable_type!(History => "worth.query.document-history.matching-history.v1");
impl
    worth_query_decl::facade::application_history::ApplicationHistoryBinding<
        DocumentRetentionSchema,
    > for History
{
    type Access = Input;
    const IDENTITY: &'static str = "MatchingDocumentRetentionHistory";
}
