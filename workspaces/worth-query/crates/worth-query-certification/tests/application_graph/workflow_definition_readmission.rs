//! Definition readmission through native checkpoint restore and cold compilation.

use super::document_retention_model::{
    host::{publish_workflow_on_first_program, restore_on_first_program, DocumentWorkflowRuntime},
    programs::{validated_second_program, OccurrenceAction, WorkflowOwnerProgram},
    retention_entry::OccurrenceSetRetentionBinding,
    schema::DocumentRetentionSchema,
    workflow::{advance_instance, propose_authoring_instance_with_retention, publish_definition},
};
use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
    },
    application_installation::{
        WorthQueryApplicationCheckpoint, WorthQueryApplicationProgramRoster,
        WorthQueryCheckpointCapturePolicy,
    },
};

fn capture(
    application: &DocumentWorkflowRuntime<
        impl worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition<
            DocumentRetentionSchema,
        >,
        impl worth_query_host::facade::declaration::application_program::ApplicationWorkflowSpec<
            Schema = DocumentRetentionSchema,
        >,
    >,
) -> Vec<u8> {
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

fn restore(
    bytes: Vec<u8>,
) -> DocumentWorkflowRuntime<
    WorkflowOwnerProgram,
    crate::document_retention_model::workflow::governed_read::GovernedDocumentWorkflow,
> {
    let program =
        worth_query_host::facade::declaration::application_program::ApplicationProgramAuthoring::<
            DocumentRetentionSchema,
            WorkflowOwnerProgram,
        >::begin()
        .validated_program()
        .unwrap();
    crate::document_retention_model::workflow::retain_workflow_for(
        super::document_retention_model::host::restore(
            program,
            WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes),
            WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        )
        .expect("the original program and native checkpoint restore before definition readmission"),
        crate::document_retention_model::workflow::workflow_resources(),
    )
}

#[test]
fn workflow_actor_contract_authentic_predecessor_refuses_before_native_restore() {
    let bytes = include_bytes!("workflow_definition_readmission/missing_input.checkpoint");
    let original = bytes.to_vec();
    let checkpoint = WorthQueryApplicationCheckpoint::from_untrusted_bytes(original.clone());
    let retained = checkpoint.clone();
    let backing = retained.bytes().as_ptr();
    let denial = restore_on_first_program(
        checkpoint,
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .err()
    .expect("displaced checkpoint must refuse before native restoration");
    match denial {
        worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial::Graph(cause) => {
            assert!(matches!(cause.kind(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::UnsupportedCheckpointFormat { observed: 9, supported: 13 }));
        }
        other => panic!("expected typed intact format refusal: {other:?}"),
    }
    assert_eq!(bytes.as_slice(), original);
    assert_eq!(retained.bytes(), original);
    assert_eq!(retained.bytes().as_ptr(), backing);
}

#[test]
fn workflow_actor_contract_reheadered_predecessor_refuses_native_frame_before_authority_readmission(
) {
    use sha2::{Digest, Sha256};
    let original = include_bytes!("workflow_definition_readmission/missing_input.checkpoint");
    assert_eq!(&original[40..42], &9u16.to_be_bytes());
    // This original export has no producer rows. The reheadered court therefore
    // cannot fail at an unrelated producer wire before reaching native recovery.
    assert_eq!(&original[58..66], &0u64.to_be_bytes());
    let native_len =
        usize::try_from(u64::from_be_bytes(original[50..58].try_into().unwrap())).unwrap();
    // Reframe only Query's envelope. The authentic old native authority layout
    // is left intact and must not acquire the new workflow interpretation.
    assert!(!original[66..66 + native_len].starts_with(b"WORTH-REL-CHK8\n"));
    let mut body = Vec::from(13u16.to_be_bytes());
    body.extend_from_slice(&2u16.to_be_bytes());
    body.extend_from_slice(&4u16.to_be_bytes());
    body.extend_from_slice(&original[42..]);
    let mut bytes = original[..8].to_vec();
    bytes.extend_from_slice(&Sha256::digest(&body));
    bytes.extend_from_slice(&body);
    assert_eq!(&bytes[70..70 + native_len], &original[66..66 + native_len]);
    let checkpoint = WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes);
    let retained = checkpoint.clone();
    let intact = retained.bytes().to_vec();
    let backing = retained.bytes().as_ptr();
    let denial = restore_on_first_program(
        checkpoint,
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .err()
    .expect("the old native frame must refuse before authority readmission");
    match denial {
        worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial::Graph(cause) => {
            // The native frame owner refuses first; schema-authority readmission
            // (which can report UnsupportedRecoveredNativeAuthorityLayout) is unreachable.
            assert_eq!(cause.kind(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected);
            assert!(cause.subject().contains("UnsupportedLegacySemantics"), "{cause:?}");
            assert!(cause.subject().contains("unsupported native checkpoint frame; CHK8 required"), "{cause:?}");
        }
        other => panic!("expected native frame refusal: {other:?}"),
    }
    assert_eq!(retained.bytes(), intact);
    assert_eq!(retained.bytes().as_ptr(), backing);
    assert_eq!(&retained.bytes()[46..], &original[42..]);
}

#[path = "workflow_definition_readmission/current.rs"]
mod current;

#[test]
fn current_draft_policy_drift_refuses_before_publication() {
    use worth_query_package_archive::facade::{
        decode_workflow_definition_draft, encode_workflow_definition_draft,
        WorthQueryPackageArchiveLimits, WorthQueryWorkflowDefinitionDraftDenialKind as Kind,
    };
    let application = publish_workflow_on_first_program();
    let definition =
        super::workflow_execution::definition::<OccurrenceSetRetentionBinding>().unwrap();
    let bytes =
        encode_workflow_definition_draft(&definition, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap();
    let head = application.runtime().current_world();
    for (field, expected) in [
        (1, Kind::ChangedObservationPolicy),
        (2, Kind::ChangedActorPolicy),
    ] {
        let altered = replace_header_text(&bytes, field, "unsupported-policy");
        let draft =
            decode_workflow_definition_draft(&altered, WorthQueryPackageArchiveLimits::DEFAULT)
                .unwrap();
        let denial = draft
            .author(application.workflow_spec())
            .err()
            .expect("changed policy refuses");
        assert_eq!(denial.kind(), expected);
        assert_eq!(application.runtime().current_world(), head);
    }
}

// WQWD's admitted current header: spec, observation policy, actor policy.
// This creates hostile current metadata, never a purported old export.
fn replace_header_text(bytes: &[u8], field: usize, replacement: &str) -> Vec<u8> {
    let mut position = 6;
    for _ in 0..field {
        let length = u32::from_be_bytes(bytes[position..position + 4].try_into().unwrap()) as usize;
        position += 4 + length;
    }
    let length = u32::from_be_bytes(bytes[position..position + 4].try_into().unwrap()) as usize;
    let mut altered = bytes[..position].to_vec();
    altered.extend_from_slice(&(replacement.len() as u32).to_be_bytes());
    altered.extend_from_slice(replacement.as_bytes());
    altered.extend_from_slice(&bytes[position + 4 + length..]);
    altered
}

#[test]
fn current_checkpoint_refusal_preserves_source_and_existing_application() {
    use sha2::{Digest, Sha256};
    use worth_query_host::facade::{
        application_installation::WorthQueryInMemoryApplicationDenial,
        primary_graph::WorthQueryPrimaryGraphInstallationDenialKind as Kind,
    };
    let application = publish_workflow_on_first_program();
    let original = capture(&application);
    let head = application.runtime().current_world();
    for marker_mismatch in [true, false] {
        let mut bytes = original.clone();
        let expected = if marker_mismatch {
            bytes[42..44].copy_from_slice(&1u16.to_be_bytes());
            let checksum = Sha256::digest(&bytes[40..]);
            bytes[8..40].copy_from_slice(&checksum);
            Kind::UnsupportedCheckpointWorkflowContract {
                facts: 1,
                instances: 4,
            }
        } else {
            *bytes.last_mut().unwrap() ^= 1;
            Kind::CheckpointRecoveryRejected
        };
        let checkpoint = WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes.clone());
        let retained = checkpoint.clone();
        let denial = restore_on_first_program(
            checkpoint,
            WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        )
        .err()
        .expect("hostile current artifact must return no runtime");
        match denial {
            WorthQueryInMemoryApplicationDenial::Graph(cause) => assert_eq!(cause.kind(), expected),
            other => panic!("must reach exact artifact refusal: {other:?}"),
        }
        assert_eq!(retained.bytes(), bytes);
        assert_eq!(application.runtime().current_world(), head);
        assert_eq!(capture(&application), original);
    }
}
