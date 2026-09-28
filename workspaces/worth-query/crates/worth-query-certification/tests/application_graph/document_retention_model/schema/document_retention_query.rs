//! The ordinary read that reports how many days a document is actually retained.
//!
//! Every court assertion about a performed or denied mutation is checked
//! against this read, so the evidence is the committed value rather than the
//! outcome variant alone.

use worth_query_host::facade::declaration;
use worth_query_host::facade::declaration::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryResultFieldRef, ApplicationQueryResultShapeBuilder,
};
use worth_query_host::facade::{
    worth_query_application_query, worth_query_portable_type, worth_query_structured_value_binding,
};

use super::{
    Document, DocumentFacts, DocumentIdentityField, DocumentRetentionField, DocumentRetentionSchema,
};

pub struct DocumentQueryParameters;
pub struct DocumentIdentitySlot;
pub struct DocumentRetentionSlot;
pub struct DocumentConditionRetentionSlot;
worth_query_portable_type!(DocumentIdentitySlot => "worth.query.certification.document-retention.identity-slot.v1");
worth_query_portable_type!(DocumentRetentionSlot => "worth.query.certification.document-retention.retention-slot.v1");
worth_query_portable_type!(DocumentConditionRetentionSlot => "worth.query.certification.document-retention.condition-retention-slot.v1");

/// One document as an ordinary read reports it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentRetentionRow {
    pub identity: String,
    pub retention_days: u64,
}
worth_query_portable_type!(DocumentRetentionRow => "worth.query.certification.document-retention.row.v1");

worth_query_structured_value_binding!(pub DocumentQueryParametersBinding for DocumentQueryParameters {
    identity: "DocumentQueryParameters"
});
worth_query_structured_value_binding!(pub DocumentRetentionRowBinding for DocumentRetentionRow {
    identity: "worth.query.certification.document-retention.row.v1"
});
worth_query_structured_value_binding!(pub DocumentRetentionConditionBinding for bool {
    identity: "worth.query.certification.document-retention.condition.v1"
});
worth_query_application_query!(
    pub DocumentRetentionQuery for DocumentRetentionSchema,
    identity "DocumentRetentionQuery",
    parameters DocumentQueryParametersBinding,
    result DocumentRetentionRowBinding,
    scope Document => "Document",
    name "document_retention_query"
);
worth_query_application_query!(
    pub DocumentRetentionConditionQuery for DocumentRetentionSchema,
    identity "DocumentRetentionConditionQuery",
    parameters DocumentQueryParametersBinding,
    result DocumentRetentionConditionBinding,
    scope Document => "Document",
    name "document_retention_condition_query"
);

pub fn document_retention_query_definition() -> ApplicationQueryDefinition<
    DocumentRetentionSchema,
    DocumentRetentionQuery,
    DocumentQueryParameters,
    DocumentRetentionRow,
    Document,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        DocumentRetentionSchema,
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
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 5))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .expect("the document retention query is canonical")
}

pub fn document_retention_condition_query_definition() -> ApplicationQueryDefinition<
    DocumentRetentionSchema,
    DocumentRetentionConditionQuery,
    DocumentQueryParameters,
    bool,
    Document,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        DocumentRetentionSchema,
        DocumentRetentionConditionQuery,
        Document,
        bool,
        DocumentRetentionConditionBinding,
    >::new(Document::reference())
    .field(condition_retention_result())
    .build();
    ApplicationQueryDefinitionBuilder::declare(DocumentRetentionConditionQuery::reference())
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
        .expect("the document condition query is canonical")
}

type ResultField<Slot, Field, Value, Write> = ApplicationQueryResultFieldRef<
    DocumentRetentionQuery,
    Slot,
    DocumentRetentionSchema,
    Document,
    DocumentFacts,
    Field,
    Value,
    Write,
    declaration::application_schema::EqualityPredicate,
    declaration::application_schema::NoApplicationUnit,
>;

pub(super) fn identity_result() -> ResultField<
    DocumentIdentitySlot,
    DocumentIdentityField,
    String,
    declaration::application_schema::ReadOnly,
> {
    ApplicationQueryResultFieldRef::new("identity", DocumentIdentityField::reference())
}

pub(super) fn retention_result() -> ResultField<
    DocumentRetentionSlot,
    DocumentRetentionField,
    u64,
    declaration::application_schema::ReadWrite,
> {
    ApplicationQueryResultFieldRef::new("retention", DocumentRetentionField::reference())
}

pub(super) fn condition_retention_result() -> ApplicationQueryResultFieldRef<
    DocumentRetentionConditionQuery,
    DocumentConditionRetentionSlot,
    DocumentRetentionSchema,
    Document,
    DocumentFacts,
    DocumentRetentionField,
    u64,
    declaration::application_schema::ReadWrite,
    declaration::application_schema::EqualityPredicate,
    declaration::application_schema::NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("retention", DocumentRetentionField::reference())
}
