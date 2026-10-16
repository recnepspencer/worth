//! Current restore lookup uses original native intent DATA and fresh disclosure authority.
use super::super::{
    host::DocumentWorkflowRuntime,
    operator_identity::{authenticate_reviewer, authenticate_ungranted_reviewer, request_scope},
    retention_entry::DOCUMENT_IDENTITY,
    schema::DocumentRetentionSchema,
};
use super::governed_read::{AuditRead, GovernedDocumentWorkflow, SubjectRead, WorkflowReadInput};
use worth_query_host::facade::{
    application_discovery::{WorthQueryWorkflowInstanceRead, WorthQueryWorkflowReadDenial},
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryWorkflowInstanceDiscoveryIntent,
        WorthQueryWorkflowObservationDenial,
    },
    declaration::application_program::ApplicationProgramDefinition,
    primary_graph::{
        WorthQueryOperationAuthorizationDenialKind, WorthQueryOperationAuthorizationRefusalKind,
    },
};

/// Both current-restore courts deliberately author the existing governed Spec.
/// The default ReviewedDocumentWorkflow keeps its disabled read policy.
pub fn read<Program: ApplicationProgramDefinition<DocumentRetentionSchema>>(
    application: &DocumentWorkflowRuntime<Program, GovernedDocumentWorkflow>,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<DocumentRetentionSchema>,
) -> WorthQueryWorkflowInstanceRead {
    let runtime = application.runtime();
    let scope = request_scope();
    let reader = authenticate_reviewer(runtime.installed_schema(), &scope);
    let basis = runtime
        .request(&reader, &scope)
        .retain_current_read()
        .unwrap();
    let denied_scope = request_scope();
    let denied = authenticate_ungranted_reviewer(runtime.installed_schema(), &denied_scope);
    let refusal = runtime
        .request(&denied, &denied_scope)
        .at(&basis)
        .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            original,
            input(),
            input(),
        )
        .err()
        .expect("original intent data cannot authorize an ungranted reader");
    assert!(
        matches!(refusal,
            WorthQueryWorkflowObservationDenial::Read(WorthQueryWorkflowReadDenial::Authorization(ref cause))
            if cause.kind() == WorthQueryOperationAuthorizationDenialKind::Refused(WorthQueryOperationAuthorizationRefusalKind::CapabilityGrantMissing)
        ),
        "{refusal:?}"
    );
    let found = runtime
        .request(&reader, &scope)
        .at(&basis)
        .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            original,
            input(),
            input(),
        )
        .expect("fresh independent subject/audit grants reacquire current restored data");
    if let WorthQueryWorkflowInstanceRead::Live { observation, .. } = &found {
        assert_eq!(observation.selected_commit(), basis.selected_commit());
    }
    found
}

pub fn input() -> WorkflowReadInput {
    WorkflowReadInput {
        document_identity: DOCUMENT_IDENTITY.to_owned(),
    }
}
