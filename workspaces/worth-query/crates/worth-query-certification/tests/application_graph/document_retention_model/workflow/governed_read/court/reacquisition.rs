//! Original intent is data across ownership loss; every lookup admits a fresh reader.
use super::super::super::super::{
    host,
    programs::{validated_second_program, RetentionProgramP1},
    workflow,
};
use super::*;
use worth_query_host::facade::application_entry::{
    PerformedWorkflowInstanceStart, PublishedWorkflowInstanceRef, WorkflowProgressOutcome,
};
use worth_query_host::facade::application_installation::{
    WorthQueryApplicationProgramRoster, WorthQueryCheckpointCapturePolicy,
};
#[path = "reacquisition/successors.rs"]
mod successors;

#[test]
fn original_intent_reacquires_carried_start_after_refs_drop_and_current_restore() {
    let mut application = install();
    support_successor(&mut application);
    let p0 = *application.vocabulary().program_revision();
    let p1 = *application
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .installed_program()
        .revision();
    assert_ne!(p0, p1);
    let definition = publish(&application);
    let abandoned = prepare(&application, definition.clone(), 92_701);
    let missing = abandoned.discovery_intent();
    drop(abandoned);
    let prepared = prepare(&application, definition, 92_702);
    let original = prepared.discovery_intent();
    let performed = started(prepared.execute());
    let entity = performed.instance().entity_id();
    let principal = performed.receipt().principal_scope().principal();
    let actor = (
        principal.partition_id(),
        principal.local_slot(),
        principal.generation(),
    );
    drop(performed); // No live instance/definition ref survives into adoption or restore.
                     // Reacquisition must not reauthorize the historical start grant.
    revoke(&application, "workflow-instance-start-grant", 92_704);
    carry(&application, &p1);
    let scope = request_scope();
    let reader = authenticate_reviewer(application.runtime().installed_schema(), &scope);
    let basis = application
        .runtime()
        .request(&reader, &scope)
        .retain_current_read()
        .unwrap();
    let found = read(&application, &reader, &scope, &basis, &original).unwrap();
    let found = match found {
        WorthQueryWorkflowInstanceRead::Live {
            instance,
            observation,
        } => {
            assert_eq!(instance.entity_id(), entity);
            assert_eq!(instance.program_revision(), &p0);
            assert_eq!(observation.selected_commit(), basis.selected_commit());
            assert_start_actor(&observation, actor);
            instance
        }
        _ => panic!("carried original intent must locate the real live native start"),
    };
    assert!(matches!(
        read(&application, &reader, &scope, &basis, &missing).unwrap(),
        WorthQueryWorkflowInstanceRead::Missing { .. }
    ));
    drop(found);
    drop(basis);
    let checkpoint = application
        .runtime()
        .with_host_advancement(|phase| {
            application
                .runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    // Clone retains the actual immutable backing supplied to restore.
    let original_bytes = checkpoint.bytes().to_vec();
    drop(application);
    let mut restored = super::super::runtime::retain(
        host::restore_on_first_program(
            checkpoint.clone(),
            WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        )
        .unwrap(),
    );
    support_successor(&mut restored);
    let fresh_scope = request_scope();
    let fresh_reader = authenticate_reviewer(restored.runtime().installed_schema(), &fresh_scope);
    let basis = restored
        .runtime()
        .request(&fresh_reader, &fresh_scope)
        .retain_current_read()
        .unwrap();
    let reacquired = match read(&restored, &fresh_reader, &fresh_scope, &basis, &original).unwrap() {
        WorthQueryWorkflowInstanceRead::Live { instance, observation } => {
            assert_eq!(instance.entity_id(), entity);
            assert_eq!(instance.program_revision(), &p0);
            assert_eq!(observation.selected_commit(), basis.selected_commit());
            assert_start_actor(&observation, actor);
            instance
        }
        _ => panic!("current restore must reacquire original start without replaying start or retaining a ref"),
    };
    assert!(matches!(
        read(&restored, &fresh_reader, &fresh_scope, &basis, &missing).unwrap(),
        WorthQueryWorkflowInstanceRead::Missing { .. }
    ));
    assert_eq!(checkpoint.bytes(), original_bytes.as_slice());
    // Original DATA remains P0; the ordinary command must freshly admit current P1.
    assert!(matches!(
        advance(&restored, reacquired, 92_703),
        WorkflowProgressOutcome::Completed(_)
    ));
}

fn support_successor(application: &mut GovernedRuntime) {
    let supported = workflow::runtime::install_workflow_spec_for::<
        RetentionProgramP1,
        GovernedDocumentWorkflow,
    >(
        application.runtime().installed_schema(),
        application
            .supported_program::<RetentionProgramP1>()
            .unwrap()
            .installed_program(),
        workflow::runtime::workflow_resources(),
    );
    application.support_workflow_spec(supported).unwrap();
}
fn carry(
    application: &GovernedRuntime,
    revision: &worth_query_host::facade::declaration::application_program::ApplicationProgramRevision,
) {
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    let programs = application.runtime().request(&principal, &scope).programs();
    let comparison = programs.compare(revision).unwrap();
    let prepared = programs.adopt(&comparison).unwrap()
        .workflow(worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::carry_compatible()).prepare(1_024).unwrap();
    assert!(matches!(prepared.publish(application.runtime(), &principal, &scope), worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPublicationOutcome::Performed(_)));
}
fn assert_start_actor(view: &WorthQueryWorkflowObservation, expected: (u32, u64, u32)) {
    match view.audit() {
        WorthQueryWorkflowAudit::Visible { start, .. } => {
            let actor = start.entity_id();
            assert_eq!(
                (
                    actor.partition_value(),
                    actor.local_slot_value(),
                    actor.generation_value()
                ),
                expected
            );
        }
        _ => panic!("independent admitted audit reader must disclose the original actor"),
    }
}
fn started(outcome: WorkflowInstanceStartOutcome) -> PerformedWorkflowInstanceStart {
    match outcome {
        WorkflowInstanceStartOutcome::Started(done) => done,
        other => panic!("genuine start must publish: {other:?}"),
    }
}
fn live(view: WorthQueryWorkflowInstanceRead) -> PublishedWorkflowInstanceRef {
    match view {
        WorthQueryWorkflowInstanceRead::Live { instance, .. } => instance,
        _ => panic!("expected a genuinely live locator"),
    }
}
fn advance(
    application: &GovernedRuntime,
    instance: PublishedWorkflowInstanceRef,
    key: u64,
) -> WorkflowProgressOutcome {
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    application
        .runtime()
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(workflow::WorkflowAdvanceIntent {
            input: workflow::WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&key)
        .prepare_workflow_advance(application.vocabulary(), instance)
        .unwrap()
        .execute()
}
