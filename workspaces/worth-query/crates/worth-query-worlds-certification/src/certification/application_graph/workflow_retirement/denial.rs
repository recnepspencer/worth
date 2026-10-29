//! Retirement admits only the spec's authoring authority, on the definition's
//! own branch, through the workflow runtime that issued the request.

pub use worth_query_host::facade::{
    application_entry::WorkflowDefinitionPreparationDenial,
    primary_graph::WorthQueryApplicationAttemptDenialKind,
};

pub use super::super::document_retention_model::retention_entry::DOCUMENT_IDENTITY;
pub use super::super::document_retention_model::workflow::{
    WorkflowInstanceStartInput, WorkflowInstanceStartIntent,
};
pub use super::*;

pub fn expect_attempt_denial(
    result: Result<(), WorthQueryWorkflowDefinitionRetirementPreparationDenial>,
    expected: WorthQueryApplicationAttemptDenialKind,
) {
    match result {
        Err(WorthQueryWorkflowDefinitionRetirementPreparationDenial::DefinitionPreparation(
            WorkflowDefinitionPreparationDenial::Attempt(attempt),
        )) => assert_eq!(attempt.kind(), expected),
        Err(other) => panic!("expected {expected:?}, got {other:?}"),
        Ok(()) => panic!("expected {expected:?}, but the retirement prepared"),
    }
}
