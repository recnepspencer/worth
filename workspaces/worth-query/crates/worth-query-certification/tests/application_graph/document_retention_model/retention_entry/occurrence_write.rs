//! An explicitly occurrence-bound binding over the same real business handler.

use super::super::schema::{
    Document, DocumentFacts, DocumentIdentityField, DocumentPrincipalBinding,
    DocumentRetentionSchema, ExternalMapping, Principal, SetRetention, SetRetentionInput,
    SetRetentionInputBinding,
};
use super::{DocumentRetentionWrittenBinding, SetRetentionDenialBinding};
use worth_query_decl::facade::{
    application_operation::{
        ApplicationWorkflowExecutionRequirements, NoApplicationMutationOutputs,
    },
    application_schema::{NoApplicationUnit, ReadOnly, U64ApplicationValueBinding},
};
use worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget;

fn document_identity(input: &SetRetentionInput) -> String {
    input.identity.clone()
}

worth_query_decl::facade::worth_query_mutation_binding!(
    pub OccurrenceSetRetentionBinding for SetRetentionInput, schema DocumentRetentionSchema,
    identity "worth.query.certification.document-retention.occurrence-set.v1",
    input SetRetentionInputBinding,
    operation SetRetention,
    result DocumentRetentionWrittenBinding,
    idempotency u64, identity "worth.query.certification.document-retention.occurrence-command.v1",
    decision WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>, denial SetRetentionDenialBinding,
    handler identity "worth.query.certification.document-retention.occurrence-handler.v1",
    workflow_authority required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::new(),
    outputs NoApplicationMutationOutputs,
    principal DocumentPrincipalBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Document, DocumentFacts, DocumentIdentityField, String, ReadOnly, NoApplicationUnit,
    field DocumentIdentityField::reference(),
    value document_identity,
    candidates creates 0, deletes 0, links 0, unlinks 0, writes 2, emits 0,
    resources retained_representation_bytes 1024
);
