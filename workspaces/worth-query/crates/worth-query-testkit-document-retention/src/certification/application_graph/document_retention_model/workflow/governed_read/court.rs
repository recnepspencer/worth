//! Real original-start lookup, independent disclosure grants and retained-data revocation.
pub use super::super::super::{
    operator_identity::{
        authenticate_operator, authenticate_reviewer, authenticate_ungranted_reviewer,
        request_scope,
    },
    retention_entry::DOCUMENT_IDENTITY,
    schema::{Document, DocumentRetentionField},
};
pub use super::super::{
    WorkflowGrantStatusInput, WorkflowGrantStatusIntent, WorkflowInstanceStartInput,
    WorkflowInstanceStartIntent, WorkflowInstanceStartOperation,
};
pub use super::*;
pub use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
    },
    application_discovery::*,
    application_entry::*,
};
#[path = "court/definition.rs"]
pub mod definition;
#[path = "court/enumeration.rs"]
pub mod enumeration;
#[path = "court/history.rs"]
pub mod history;
#[path = "court/history_stimuli.rs"]
pub mod history_stimuli;
#[path = "court/provenance.rs"]
pub mod provenance;
#[path = "court/publication.rs"]
pub mod publication;
#[path = "court/reacquisition.rs"]
pub mod reacquisition;
pub use publication::{prepare, publish, revoke};

pub fn read(
    application: &GovernedRuntime,
    principal: &WorthQueryAuthenticatedExternalPrincipal<DocumentRetentionSchema>,
    scope: &WorthQueryRequestScope,
    observation: &WorthQueryApplicationReadObservation,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<DocumentRetentionSchema>,
) -> Result<WorthQueryWorkflowInstanceRead, WorthQueryWorkflowObservationDenial> {
    application
        .runtime()
        .request(principal, scope)
        .at(observation)
        .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            original,
            input(),
            input(),
        )
}
pub fn input() -> WorkflowReadInput {
    WorkflowReadInput {
        document_identity: DOCUMENT_IDENTITY.to_owned(),
    }
}
