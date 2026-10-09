//! The authored meaning both document-retention programs are written against.
//!
//! One schema installs one document entity carrying a retention period in
//! days, one operation that sets it, one ordinary read, and BOTH
//! document-retention rule contracts. Which of the two actually decides a
//! candidate is not a property of this schema: it is decided by the program
//! the occurrence is running.

use worth_query_host::facade::{declaration, primary_graph};
use worth_query_host::facade::{
    worth_query_application, worth_query_application_contribution, worth_query_aspect,
    worth_query_entity, worth_query_field, worth_query_operation, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_portable_type, worth_query_principal_binding,
    worth_query_relation, worth_query_structured_value_binding,
};

#[path = "schema/document_retention_invariants.rs"]
mod document_retention_invariants;
#[path = "schema/document_retention_query.rs"]
mod document_retention_query;

pub use document_retention_invariants::{
    DocumentRetentionV1, DocumentRetentionV2, DocumentRetentionV3,
};
pub use document_retention_query::{
    document_retention_condition_query_definition, document_retention_query_definition,
    DocumentQueryParameters, DocumentQueryParametersBinding, DocumentRetentionConditionBinding,
    DocumentRetentionConditionQuery, DocumentRetentionQuery, DocumentRetentionRow,
    DocumentRetentionRowBinding,
};

worth_query_application! {
    pub DocumentRetentionSchema {
        owner: "document_retention_courtroom",
        version: (1, 0),
        contributions: [DocumentRetentionContribution],
    }
}

worth_query_application_contribution! {
    pub contribution DocumentRetentionContribution in DocumentRetentionSchema {
        identity: "document_retention_courtroom.application_graph.v1",
        members: |schema| {
            let schema = schema
                .entity(ExternalMapping::reference())
                .entity(Principal::reference())
                .entity(Document::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalFacts::reference())
                .aspect(Document::reference(), DocumentFacts::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .field(Document::reference(), DocumentIdentityField::reference())
                .field(Document::reference(), DocumentRetentionField::reference())
                .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
                .principal_binding(DocumentPrincipalBinding::reference())
                .operation(
                    SetRetention::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )

                .operation_projection_work_budget(SetRetention::reference(), 8)
                .operation_read_field(SetRetention::reference(), DocumentIdentityField::reference())
                .operation_read_field(SetRetention::reference(), DocumentRetentionField::reference())
                .operation_write(SetRetention::reference(), DocumentRetentionField::reference())
                .invariant(document_retention_invariants::first_definition())
                .invariant(document_retention_invariants::second_definition())
                .application_query(document_retention_query_definition())
                .application_query(document_retention_condition_query_definition())
                .application_query(super::retention_days::document_retention_days_query_definition());
            super::workflow::declare(super::assessment_output::declare(super::retention_days::declare(
                super::retention_entry::declare(schema),
            )))
        }
    }
}

worth_query_entity!(pub ExternalMapping for DocumentRetentionSchema);
worth_query_entity!(pub Principal for DocumentRetentionSchema);
worth_query_entity!(pub Document for DocumentRetentionSchema);
worth_query_aspect!(pub ExternalIdentity for DocumentRetentionSchema, ExternalMapping; identity = AspectIdentity(0x91750101), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PrincipalFacts for DocumentRetentionSchema, Principal; identity = AspectIdentity(0x91750102), revision = AspectContractRevision(1),);
worth_query_aspect!(pub DocumentFacts for DocumentRetentionSchema, Document; identity = AspectIdentity(0x91750103), revision = AspectContractRevision(1),);
worth_query_field!(pub ExternalIdentityField for DocumentRetentionSchema, ExternalMapping, ExternalIdentity: declaration::authentication::WorthQueryExternalPrincipalIdentity => declaration::authentication::WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for DocumentRetentionSchema, ExternalMapping, ExternalIdentity: declaration::authentication::WorthQueryPrincipalMappingStatus => declaration::authentication::WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentityField for DocumentRetentionSchema, Principal, PrincipalFacts: u64 => declaration::application_schema::U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub DocumentIdentityField for DocumentRetentionSchema, Document, DocumentFacts: String => declaration::application_schema::StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub DocumentRetentionField for DocumentRetentionSchema, Document, DocumentFacts: u64 => declaration::application_schema::U64ApplicationValueBinding, read_write, equality);
worth_query_relation!(pub MappingTarget in DocumentRetentionSchema, ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(
    pub DocumentPrincipalBinding in DocumentRetentionSchema,
    mapping ExternalMapping {
        identity: ExternalIdentityField,
        status: MappingStatusField,
        target: MappingTarget => Principal,
        principal_identity: PrincipalIdentityField
    }
);

/// The exact retention one request asks a named document to carry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetRetentionInput {
    pub identity: String,
    pub retention_days: u64,
}

thread_local! {
    static SET_RETENTION_INPUT_ENCODINGS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

impl SetRetentionInput {
    /// Starts counting how many times this thread serializes a retention input.
    pub fn reset_encoding_count() {
        SET_RETENTION_INPUT_ENCODINGS.with(|count| count.set(0));
    }

    /// How many times this thread serialized a retention input since the reset.
    pub fn encoding_count() -> u32 {
        SET_RETENTION_INPUT_ENCODINGS.with(std::cell::Cell::get)
    }
}

/// Serializes exactly as a derived impl would, and counts each call so a proof
/// can show how many times a request encodes its input.
impl serde::Serialize for SetRetentionInput {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        SET_RETENTION_INPUT_ENCODINGS.with(|count| count.set(count.get() + 1));
        let mut state = serializer.serialize_struct("SetRetentionInput", 2)?;
        state.serialize_field("identity", &self.identity)?;
        state.serialize_field("retention_days", &self.retention_days)?;
        state.end()
    }
}
worth_query_portable_type!(SetRetentionInput => "worth.query.certification.document-retention.set-input.v1");
worth_query_structured_value_binding!(pub SetRetentionInputBinding for SetRetentionInput {
    identity: "worth.query.certification.document-retention.set-input.v1"
});
worth_query_operation!(pub SetRetention for DocumentRetentionSchema, input SetRetentionInputBinding);
worth_query_operation_reads!(SetRetention => [DocumentIdentityField, DocumentRetentionField]);
worth_query_operation_writes!(SetRetention => [DocumentRetentionField]);

impl primary_graph::WorthQueryApplicationProjection<DocumentRetentionSchema, DocumentRetentionQuery>
    for DocumentRetentionRow
{
    fn project(
        row: &primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            DocumentRetentionSchema,
            DocumentRetentionQuery,
        >,
    ) -> Result<Self, primary_graph::WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(document_retention_query::identity_result())?,
            retention_days: row.field(document_retention_query::retention_result())?,
        })
    }
}

impl
    primary_graph::WorthQueryApplicationProjection<
        DocumentRetentionSchema,
        DocumentRetentionConditionQuery,
    > for bool
{
    fn project(
        row: &primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            DocumentRetentionSchema,
            DocumentRetentionConditionQuery,
        >,
    ) -> Result<Self, primary_graph::WorthQueryApplicationProjectionDenial> {
        Ok(row.field(document_retention_query::condition_retention_result())? > 0)
    }
}
