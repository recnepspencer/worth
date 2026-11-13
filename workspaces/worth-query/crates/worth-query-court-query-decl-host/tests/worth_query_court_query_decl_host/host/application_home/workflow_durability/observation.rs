//! Fresh governed discovery obtains current locators from original start DATA.
use super::*;
pub(super) fn read<P>(
    home: &Home<P>,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<document::schema::DocumentRetentionSchema>,
) -> WorthQueryWorkflowInstanceRead {
    let runtime = home.runtime().runtime();
    let scope = request_scope();
    let principal = authenticate_reviewer(runtime.installed_schema(), &scope);
    let request = runtime.request(&principal, &scope);
    let basis = request.retain_current_read().unwrap();
    request
        .at(&basis)
        .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            home.vocabulary(),
            original,
            input(),
            input(),
        )
        .unwrap()
}
pub(super) fn input() -> WorkflowReadInput {
    WorkflowReadInput {
        document_identity: DOCUMENT_IDENTITY.to_owned(),
    }
}
pub(super) fn instance(
    home: &Home<
        impl worth_query_decl::facade::application_program::ApplicationProgramDefinition<
            document::schema::DocumentRetentionSchema,
        >,
    >,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<document::schema::DocumentRetentionSchema>,
) -> PublishedWorkflowInstanceRef {
    match read(home, original) {
        WorthQueryWorkflowInstanceRead::Live { instance, .. } => instance,
        _ => panic!("the original workflow remains live"),
    }
}
pub(super) fn proposal(
    home: &Home<
        impl worth_query_decl::facade::application_program::ApplicationProgramDefinition<
            document::schema::DocumentRetentionSchema,
        >,
    >,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<document::schema::DocumentRetentionSchema>,
) -> PublishedWorkflowProposalRef {
    let read = read(home, original);
    let runtime = home.runtime().runtime();
    let scope = request_scope();
    let principal = authenticate_reviewer(runtime.installed_schema(), &scope);
    let request = runtime.request(&principal, &scope);
    let basis = request.retain_current_read().unwrap();
    let page = request
        .at(&basis)
        .read_workflow_history::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
            home.vocabulary(),
            read.target().unwrap(),
            input(),
            input(),
            16,
            WorthQueryWorkflowIndexRead::PublishedOnly,
            None,
        )
        .unwrap();
    page.entries()
        .iter()
        .find_map(|entry| entry.proposal().cloned())
        .expect("original proposal is retained")
}
