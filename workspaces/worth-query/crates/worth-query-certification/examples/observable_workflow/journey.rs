//! Real requests advance one installed workflow; waits retain native custody.
use crate::document_retention_model::{
    operator_identity::*, retention_entry::DOCUMENT_IDENTITY, schema::*, workflow::*,
    MixedApplication,
};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use worth_query_host::facade::application_entry::*;

pub fn run() {
    // Bootstrap runs before any workflow request/progress frame is active.
    let application = crate::document_retention_model::installation::publish();
    // All original native owners and references die when execute returns.
    let captured = execute(application);
    crate::restoration::inspect(captured);
}

// Keep the workflow request frame separate from bootstrap and restoration.
#[inline(never)]
fn execute(application: MixedApplication) -> crate::restoration::CapturedJourney {
    let runtime = application.runtime();
    let directory = std::env::temp_dir().join(format!(
        "worth-observable-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let archive = Arc::new(crate::archive::ArchiveTransport::new(directory.clone()));
    runtime
        .install_external_effect_transport(archive.clone())
        .expect("install the actual archive transport");
    let operation = runtime
        .installed_schema()
        .installed_operation(crate::effect::ArchiveOperation::reference())
        .unwrap();
    let verifier = runtime
        .install_inbound_occurrence_verifier(&operation, archive.clone())
        .unwrap();
    let scope = request_scope();
    let actor = authenticate_operator(runtime.installed_schema(), &scope);
    let definition = crate::definition::definition();
    let contract = application
        .workflow_spec()
        .bind_definition(definition)
        .unwrap();
    let published = runtime
        .request(&actor, &scope)
        .mutate(WorkflowDefinitionAuthoringIntent { input: input() })
        .without_source()
        .idempotency(&1_u64)
        .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)
        .unwrap()
        .execute();
    let definition = match published {
        WorkflowDefinitionPublicationOutcome::Published(published) => {
            published.definition().clone()
        }
        other => panic!("definition publication: {other:?}"),
    };
    let prepared = runtime
        .request(&actor, &scope)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                document_identity: DOCUMENT_IDENTITY.into(),
            },
        })
        .without_source()
        .idempotency(&2_u64)
        .prepare_workflow_instance_start(&application, definition.clone())
        .unwrap();
    let original_start = prepared.discovery_intent();
    crate::discovery::inspect_missing(&application, &original_start);
    let (instance, expected_start) = match prepared.execute() {
        WorkflowInstanceStartOutcome::Started(started) => {
            let principal = started.receipt().principal_scope().principal();
            (
                started.instance().clone(),
                [
                    u64::from(principal.partition_id()),
                    principal.local_slot(),
                    u64::from(principal.generation()),
                ],
            )
        }
        other => panic!("original native start: {other:?}"),
    };
    // compiled-example: begin
    let waiting = advance(&application, &instance, &[10, 11]);
    let WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingProposal(need)) =
        waiting.stop()
    else {
        panic!("the first node must expose its actual proposal need: {waiting:?}");
    };
    assert_eq!(need.node_path(), "propose");
    assert_eq!(
        need.operation(),
        WorkflowDefinitionAuthoringOperation::reference().name()
    );
    assert_eq!(
        need.input_type(),
        WorkflowDefinitionAuthoringOperation::reference()
            .input_identity()
            .as_str()
    );
    assert!(waiting.transitions().is_empty());
    let proposal = runtime
        .request(&actor, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowDefinitionAuthoringIntent { input: input() })
        .without_source()
        .idempotency(&20_u64)
        .prepare_workflow_proposal(&application, instance.clone())
        .unwrap()
        .execute();
    let (proposal, expected_proposer) = match proposal {
        WorkflowProposalOutcome::Published(published) => {
            let principal = published.receipt().principal_scope().principal();
            (
                published.proposal().clone(),
                [
                    u64::from(principal.partition_id()),
                    principal.local_slot(),
                    u64::from(principal.generation()),
                ],
            )
        }
        other => panic!("original proposal publication: {other:?}"),
    };
    // compiled-example: end
    let capture = operation_need(advance(&application, &instance, &[30, 31, 32]), "capture");
    crate::operations::capture(&application, capture);
    assert_eq!(
        crate::document_retention_model::readback::read_retention(runtime, instance.branch()),
        9
    );
    let condition = advance(&application, &instance, &[48]);
    let WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingCondition(
        required,
    )) = condition.stop()
    else {
        panic!("the native cursor must require the real retention condition: {condition:?}");
    };
    assert_eq!(required.node_path(), "positive-retention");
    crate::operations::condition(&application, &instance, required);
    // Each assessment retains, drives and closes its actual output-demand owner.
    for (path, run_keys, demand_key, acceptance_key) in [
        ("checks/first", [50, 51], 52, 53),
        ("checks/confirmation", [54, 55], 56, 57),
    ] {
        let progress = advance(&application, &instance, &run_keys);
        assert!(
            matches!(
                progress.stop(),
                WorthQueryOrdinaryWorkflowRunStop::Outcome(
                    WorkflowProgressOutcome::AwaitingAssessment(required)
                ) if required.node_path() == path
            ),
            "the native cursor must require the exact assessment: {progress:?}"
        );
        crate::operations::assess(&application, &instance, path, demand_key, acceptance_key);
    }
    let progress = advance(&application, &instance, &[58, 59]);
    let WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingApproval(
        required,
    )) = progress.stop()
    else {
        panic!("a true native condition must expose the authored approval: {progress:?}");
    };
    let expected_approver = crate::history::observe_approval(
        &application,
        &instance,
        &original_start,
        &proposal,
        expected_proposer,
        || approve(&application, &instance, required, &proposal),
    );
    // A new admitted request reacquires the exact original live instance.
    crate::discovery::inspect(&application, &definition, &instance, &original_start);
    let required = operation_need(advance(&application, &instance, &[70, 71, 72]), "archive");
    crate::operations::archive(&application, &instance, required, &archive, &verifier);
    let finished = advance(&application, &instance, &[100, 101, 102, 103]);
    assert!(
        matches!(finished.stop(), WorthQueryOrdinaryWorkflowRunStop::Terminal),
        "the native inbound successor and terminal must finish: {finished:?}"
    );
    assert!(finished
        .transitions()
        .iter()
        .any(|transition| transition.node_path() == "archive-completion"));
    crate::restoration::assert_archive(&directory);
    let checkpoint = runtime.with_host_advancement(|phase| {
        runtime.capture_application_checkpoint(
            &phase,
            worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
        ).expect("capture current native terminal truth after original operation cleanup")
    }).expect("installed host capture opens");
    crate::restoration::CapturedJourney {
        checkpoint,
        original_start,
        expected_start,
        expected_proposer,
        expected_approver,
        proposal_identity: proposal.identity().to_owned(),
        directory,
    }
}

pub fn input() -> SetRetentionInput {
    SetRetentionInput {
        identity: DOCUMENT_IDENTITY.into(),
        retention_days: 9,
    }
}
pub fn advance(
    application: &MixedApplication,
    instance: &PublishedWorkflowInstanceRef,
    keys: &[u64],
) -> WorthQueryOrdinaryWorkflowRunProgress {
    let scope = request_scope();
    let actor = authenticate_operator(application.runtime().installed_schema(), &scope);
    application
        .runtime()
        .request(&actor, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.into(),
            },
        })
        .run_workflow(application, instance.clone())
        .idempotency_keys(keys)
        .execute()
}
fn operation_need(
    progress: WorthQueryOrdinaryWorkflowRunProgress,
    path: &str,
) -> RequiredWorkflowOperation {
    match progress.stop() {
        WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingOperation(
            required,
        )) => {
            assert_eq!(required.node_path(), path);
            required.clone()
        }
        _ => panic!("native operation need {path}: {progress:?}"),
    }
}
fn approve(
    application: &MixedApplication,
    instance: &PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowApproval,
    proposal: &PublishedWorkflowProposalRef,
) -> [u64; 3] {
    let scope = request_scope();
    let reviewer = authenticate_reviewer(application.runtime().installed_schema(), &scope);
    let signing = application
        .runtime()
        .request(&reviewer, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowApprovalIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.into(),
            },
        })
        .without_source()
        .idempotency(&60_u64)
        .prepare_workflow_approval(
            application,
            instance.clone(),
            required,
            proposal,
            WorkflowApprovalDecision::Approve,
        )
        .unwrap();
    let intent = signing
        .authentication_intent()
        .cloned()
        .expect("fresh distinct-reviewer approval requires its real challenge");
    let event = block_on(
        application
            .authentication
            .authenticate((), &reviewer, intent, &scope),
    )
    .unwrap();
    let result = signing.sign(&event).unwrap().execute();
    match result {
        WorkflowProgressOutcome::Completed(done) => {
            let actor = done
                .approval()
                .expect("actual performed approval")
                .approver();
            [
                actor.partition_value_u64(),
                actor.local_slot_value(),
                u64::from(actor.generation_value()),
            ]
        }
        other => panic!("admitted reviewer approval: {other:?}"),
    }
}
