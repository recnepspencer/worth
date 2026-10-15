//! A later authenticated request discovers the original start at one exact data basis.
//!
//! Metadata and original-start lookup use fresh admission. The history module shows
//! independent audit and native live marks; neither view substitutes for permission.
use crate::{
    definition::MixedWorkflow,
    document_retention_model::{
        operator_identity::{authenticate_operator, request_scope},
        retention_entry::DOCUMENT_IDENTITY,
        schema::DocumentRetentionSchema,
        workflow::governed_read::{AuditRead, MetadataRead, SubjectRead, WorkflowReadInput},
        MixedApplication,
    },
};
use worth_query_decl::facade::application_program::{
    ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowDefinitionReadTarget,
};
use worth_query_host::facade::{
    application_discovery::{
        WorthQueryWorkflowDefinitionDiscovery, WorthQueryWorkflowInstanceRead,
    },
    application_entry::{
        PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef,
        WorthQueryApplicationRequestExt, WorthQueryWorkflowInstanceDiscoveryIntent,
    },
};

/// Called while the native instance remains live; no original request or grant is retained.
pub fn inspect(
    application: &MixedApplication,
    definition: &PublishedWorkflowDefinitionRef,
    instance: &PublishedWorkflowInstanceRef,
    original_start: &WorthQueryWorkflowInstanceDiscoveryIntent<DocumentRetentionSchema>,
) {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch());
    let basis = request
        .retain_current_read()
        .expect("retain the later request's exact basis");
    let read = request.at(&basis);
    let target = ApplicationWorkflowDefinitionReadTarget::<MixedWorkflow>::new(
        ApplicationWorkflowDefinitionIdentity::new("observable-document-archive")
            .expect("the authored definition identity"),
    );
    let metadata = read
        .discover_workflow_definition::<MixedWorkflow, MetadataRead>(
            application.vocabulary(),
            &target,
            input(),
        )
        .expect("fresh independent metadata admission");
    assert_eq!(metadata.basis().selected_commit(), basis.selected_commit());
    match metadata.definition() {
        WorthQueryWorkflowDefinitionDiscovery::Current(found) => assert_eq!(found, definition),
        _ => panic!("the original definition remains current at this basis"),
    }
    match read
        .read_original_workflow::<MixedWorkflow, SubjectRead, AuditRead>(
            application.vocabulary(),
            original_start,
            input(),
            input(),
        )
        .expect("fresh subject admission for the original native start")
    {
        WorthQueryWorkflowInstanceRead::Live {
            instance: found,
            observation,
        } => {
            assert_eq!(&found, instance);
            assert_eq!(observation.selected_commit(), basis.selected_commit());
        }
        _ => panic!("the original start must resolve to the same live instance"),
    }
}

fn input() -> WorkflowReadInput {
    WorkflowReadInput {
        document_identity: DOCUMENT_IDENTITY.to_owned(),
    }
}
