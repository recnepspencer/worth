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
use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstancePreparationDenial, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
        WorkflowProposalOutcome, WorthQueryWorkflowInstancePreparationDenial,
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

#[test]
fn workflow_actor_contract_authentic_predecessor_refuses_before_native_restore() {
    let bytes = include_bytes!("workflow_definition_readmission/missing_input.checkpoint");
    let original = bytes.to_vec();
    let denial = restore_on_first_program(
        WorthQueryApplicationCheckpoint::from_untrusted_bytes(original.clone()),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .err()
    .expect("displaced checkpoint must refuse before native restoration");
    match denial {
        worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial::Graph(cause) => {
            assert!(matches!(cause.kind(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::UnsupportedCheckpointFormat { observed: 9, supported: 10 }));
        }
        other => panic!("expected typed intact format refusal: {other:?}"),
    }
    assert_eq!(bytes.as_slice(), original);
}

#[test]
fn workflow_actor_contract_reheadered_predecessor_refuses_native_authority_layout() {
    use sha2::{Digest, Sha256};
    let original = include_bytes!("workflow_definition_readmission/missing_input.checkpoint");
    // Reframe only Query's envelope. The authentic old native authority layout
    // is left intact and must not acquire the new workflow interpretation.
    let mut body = Vec::from(10u16.to_be_bytes());
    body.extend_from_slice(&2u16.to_be_bytes());
    body.extend_from_slice(&3u16.to_be_bytes());
    body.extend_from_slice(&original[42..]);
    let mut bytes = original[..8].to_vec();
    bytes.extend_from_slice(&Sha256::digest(&body));
    bytes.extend_from_slice(&body);
    let retained = bytes.clone();
    let denial = restore_on_first_program(
        WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .err()
    .expect("old native authority layout must refuse under a current envelope");
    match denial {
        worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial::Graph(cause) => {
            assert_eq!(cause.kind(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::UnsupportedRecoveredNativeAuthorityLayout);
        }
        other => panic!("expected native authority layout refusal: {other:?}"),
    }
    assert_eq!(&retained[46..], &original[42..]);
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
    let rebuilt = published.held_on(restored.runtime().current_world());
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
