//! Current input cardinality through ordinary publication and archive import.

use super::document_retention_model::{
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    retention_entry::OccurrenceSetRetentionBinding,
    workflow::{
        authoring_intent, definition_limits, ReviewedDocumentWorkflow,
        WorkflowDefinitionAuthoringOperation,
    },
};
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::{
    application_discovery::WorthQueryWorkflowDefinitionDiscovery, application_entry::*,
};
use worth_query_package_archive::facade::{
    decode_workflow_definition_draft, WorthQueryPackageArchiveLimits,
};

pub(super) fn draft(inputs: usize) -> AuthoredWorkflowDefinition<ReviewedDocumentWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedDocumentWorkflow>::new(
        "input-cardinality",
        definition_limits(),
    )
    .unwrap();
    let first = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("first", false)
        .unwrap();
    let second = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("second", false)
        .unwrap();
    let perform = builder
        .operation_binding::<OccurrenceSetRetentionBinding>("perform")
        .unwrap();
    let done = builder.terminal("done").unwrap();
    use ApplicationWorkflowControlOutcome::Completed;
    builder
        .start(&first)
        .control(&first, Completed, &second)
        .control(&second, Completed, &perform)
        .control(&perform, Completed, &done);
    if inputs >= 1 {
        builder.operation_input(&first, &perform);
    }
    if inputs >= 2 {
        builder.operation_input(&second, &perform);
    }
    builder.finish().unwrap()
}

#[test]
fn ordinary_publication_refuses_zero_and_two_inputs_before_publication() {
    let application = publish_workflow_on_first_program();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    for inputs in [0, 2] {
        runtime
            .with_host_advancement(|phase| {
                let refusal = runtime
                    .request(&principal, &scope)
                    .mutate(authoring_intent())
                    .workflow(&phase, &application, draft(inputs));
                match refusal {
                    Err(WorthQueryOrdinaryWorkflowPublicationDenial::InvalidDefinition(denial)) => {
                        assert_eq!(
                            denial.kind(),
                            ApplicationWorkflowValidationDenialKind::MissingOperationInput
                        );
                        assert_eq!(denial.subject(), "perform");
                    }
                    _ => panic!("ordinary publication must refuse the input contract"),
                }
            })
            .unwrap();
    }
    let policy = crate::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&policy);
    let selected = runtime
        .with_host_advancement(|phase| {
            runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
        })
        .unwrap();
    assert_eq!(
        selected
            .discover_workflow_definition::<ReviewedDocumentWorkflow>(
                &execution,
                &ApplicationWorkflowDefinitionIdentity::new("input-cardinality").unwrap()
            )
            ,
        Err(worth_query_host::facade::application_discovery::WorthQueryWorkflowDefinitionDiscoveryDenial::ObservationDisabled)
    );
    let published = runtime
        .with_host_advancement(|phase| {
            runtime
                .request(&principal, &scope)
                .mutate(authoring_intent())
                .workflow(&phase, &application, draft(1))
                .unwrap()
                .publish(WorkflowDefinitionExpectedPredecessor::Absent)
                .idempotency(&94101_u64)
        })
        .unwrap()
        .execute()
        .unwrap();
    assert!(matches!(
        published,
        WorkflowDefinitionPublicationOutcome::Published(_)
    ));
}

#[test]
fn authentic_predecessor_draft_cannot_import_executable_meaning() {
    let application = publish_workflow_on_first_program();
    // Captured with the original production validator and writer, before the
    // guarded-input requirement was enforced. No bytes are converted on import.
    let bytes = include_bytes!("workflow_definition_readmission/missing_input.wqwd");
    let original = bytes.to_vec();
    let denial = decode_workflow_definition_draft(bytes, WorthQueryPackageArchiveLimits::DEFAULT)
        .err()
        .expect("displaced authored format is refused before executable interpretation");
    assert_eq!(denial.kind(), worth_query_package_archive::facade::WorthQueryPackageArchiveDenialKind::UnsupportedWorkflowDefinitionDraftVersion);
    assert_eq!(bytes.as_slice(), original);
}
