//! Definition readmission through native checkpoint restore and cold compilation.

use super::document_retention_model::{
    host::{
        publish_workflow_on_first_program, restore_on_first_program, DocumentWorkflowRuntime,
        SEED_RETENTION,
    },
    programs::validated_second_program,
    readback::read_retention,
    retention_entry::OccurrenceSetRetentionBinding,
    workflow::{
        advance_instance, propose_authoring_instance_with_retention, publish_definition,
        retain_workflow, start_instance, ReviewedDocumentWorkflow,
    },
};
use worth_query_decl::facade::application_program::ApplicationWorkflowDefinitionIdentity;
use worth_query_host::facade::{
    application_discovery::WorthQueryWorkflowDefinitionDiscovery,
    application_entry::{
        PublishedWorkflowDefinitionRef, WorkflowDefinitionExpectedPredecessor,
        WorkflowDefinitionPublicationOutcome, WorkflowInstancePreparationDenial,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
        WorthQueryWorkflowInstancePreparationDenial,
    },
    application_installation::{
        WorthQueryApplicationCheckpoint, WorthQueryApplicationProgramRoster,
        WorthQueryCheckpointCapturePolicy,
    },
    primary_graph::WorthQueryApplicationAttemptDenialKind,
};

fn capture(application: &DocumentWorkflowRuntime) -> Vec<u8> {
    application
        .with_host_advancement(|phase| {
            application
                .runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
                .bytes()
                .to_vec()
        })
        .unwrap()
}

fn restore(bytes: Vec<u8>) -> DocumentWorkflowRuntime {
    retain_workflow(
        restore_on_first_program(
            WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes),
            WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        )
        .expect("the original program and native checkpoint restore before definition readmission"),
    )
}

fn discover(
    application: &DocumentWorkflowRuntime,
    identity: &str,
) -> PublishedWorkflowDefinitionRef {
    let policy = crate::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&policy);
    let selected = application
        .with_host_advancement(|phase| {
            application
                .runtime()
                .on_branch(application.runtime().current_world())
                .select(&phase)
                .unwrap()
        })
        .unwrap();
    match selected
        .discover_workflow_definition::<ReviewedDocumentWorkflow>(
            &execution,
            &ApplicationWorkflowDefinitionIdentity::new(identity).unwrap(),
        )
        .unwrap()
    {
        WorthQueryWorkflowDefinitionDiscovery::Current(definition) => definition,
        other => panic!("the native stored publication must be discoverable: {other:?}"),
    }
}

#[test]
fn authentic_predecessor_is_refused_at_native_input_readmission_without_rewriting_facts() {
    // Captured in this worktree before adding the guarded-input validator:
    // the production builder validated an occurrence-only operation without an
    // input edge, installed binding published it, and native capture wrote these
    // bytes. This is an original writer artifact, not mutated current facts.
    let bytes = include_bytes!("workflow_definition_readmission/missing_input.checkpoint");
    let original = bytes.to_vec();
    let application = restore(original.clone());
    let published = discover(&application, "unsupported-input-predecessor");
    let identity = published.content_identity().clone();
    let before = capture(&application);
    match start_instance(&application, published, 94002) {
        Err(WorthQueryWorkflowInstancePreparationDenial::InstancePreparation(
            WorkflowInstancePreparationDenial::Attempt(denial),
        )) => {
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionMissingOperationInput
            );
            assert_eq!(denial.subject(), "perform");
        }
        other => {
            panic!("native input admission must refuse before instance publication: {other:?}")
        }
    }
    assert_eq!(
        capture(&application),
        before,
        "refusal changes no durable facts"
    );
    assert_eq!(
        discover(&application, "unsupported-input-predecessor").content_identity(),
        &identity
    );
    assert_eq!(
        read_retention(application.runtime(), application.runtime().current_world()),
        SEED_RETENTION
    );
    assert_eq!(
        bytes.as_slice(),
        original,
        "unsupported original bytes remain intact"
    );
}

#[test]
fn current_contract_restores_original_identity_and_issues_fresh_operation_authority() {
    let application = publish_workflow_on_first_program();
    let definition =
        super::workflow_execution::definition::<OccurrenceSetRetentionBinding>().unwrap();
    let identity = definition.content_identity().clone();
    let published = match publish_definition(
        &application,
        definition,
        WorkflowDefinitionExpectedPredecessor::Absent,
        94011,
    )
    .unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("current definition must publish: {other:?}"),
    };
    assert_eq!(published.content_identity(), &identity);
    let restored = restore(capture(&application));
    let rebuilt = discover(&restored, "information-recording");
    assert_eq!(rebuilt.content_identity(), &identity);
    let instance = match start_instance(&restored, rebuilt, 94012).unwrap() {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("current native definition must start: {other:?}"),
    };
    assert!(matches!(
        propose_authoring_instance_with_retention(&restored, instance.clone(), 94013, 8).unwrap(),
        WorkflowProposalOutcome::Published(_)
    ));
    assert!(matches!(
        advance_instance(&restored, instance, 94014).unwrap(),
        WorkflowProgressOutcome::AwaitingOperation(_)
    ));
}
