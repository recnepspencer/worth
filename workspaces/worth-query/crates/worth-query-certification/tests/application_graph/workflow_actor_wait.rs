//! An actor wait is an observation-time result of a real permission denial.

use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryApplicationRequestMutationDenial,
    WorthQueryOrdinaryWorkflowRunStop, WorthQueryWorkflowAdvancePreparationDenial,
};
use worth_query_execution::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind;
use worth_query_execution::facade::workflow_advance::RequiredWorkflowActorNodeKind;

use super::bounded_dimension_model::{
    dimension_entry::PART_IDENTITY,
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    workflow::{
        advance_instance, condition_terminal_definition, propose_authoring_instance,
        publish_definition, run_instance, start_instance, terminal_definition,
        WorkflowAdvanceInput, WorkflowAdvanceIntent, WorkflowApprovalIntent, WorkflowGrantStatusInput,
        WorkflowGrantStatusIntent,
    },
};

#[test]
fn revoked_advance_grant_yields_actor_wait_only_for_the_live_instance() {
    let application = publish_workflow_on_first_program();
    let published = match publish_definition(
        &application,
        condition_terminal_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        9_217_000,
    )
    .expect("workflow definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("workflow definition did not publish: {other:?}"),
    };
    let instance = match start_instance(&application, published.definition().clone(), 9_217_001)
        .expect("workflow instance prepares")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("workflow instance did not start: {other:?}"),
    };
    assert!(matches!(
        propose_authoring_instance(&application, instance.clone(), 9_217_002),
        Ok(WorkflowProposalOutcome::Published(_))
    ));
    assert!(matches!(
        advance_instance(&application, instance.clone(), 9_217_003),
        Ok(WorkflowProgressOutcome::AwaitingCondition(_))
    ));

    change_advance_grant(&application, &instance, "revoked", 9_217_004);
    let direct = advance_instance(&application, instance.clone(), 9_217_005);
    let Err(WorthQueryWorkflowAdvancePreparationDenial::AwaitingActor(required)) = direct else {
        panic!("revoked actor must receive a typed preparation-time wait: {direct:?}");
    };
    assert_eq!(required.instance(), instance.entity_id());
    assert_eq!(required.node_path(), "positive-dimension");
    assert_eq!(required.node_kind(), RequiredWorkflowActorNodeKind::Condition);
    assert!(required.occurrence() > 0);
    assert!(!required.transition_identity().is_empty());
    assert!(required.denial().causes().iter().all(|cause| matches!(
        cause,
        WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing
            | WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing
    )));
    change_grant(&application, &instance, "workflow-approval-grant", "revoked", 9_217_011);
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    let wrong_binding = application.runtime()
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowApprovalIntent {
            input: WorkflowAdvanceInput { part_identity: PART_IDENTITY.to_owned() },
        })
        .without_source()
        .idempotency(&9_217_012)
        .prepare_workflow_advance(&application, instance.clone());
    assert!(matches!(
        wrong_binding,
        Err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission(
            WorthQueryApplicationRequestMutationDenial::Authorization(ref denial)
        )) if denial.causes().iter().any(|cause| matches!(
            cause,
            WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing
                | WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing
        ))
    ), "a denied non-advance capability cannot become an actor wait");
    let runtime = application.runtime();
    let sibling = runtime
        .branches()
        .fork(instance.branch())
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("historical sibling publishes");
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let foreign = runtime
        .request(&principal, &scope)
        .on_branch(sibling)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&9_217_009)
        .prepare_workflow_advance(&application, instance.clone());
    assert!(
        matches!(
            foreign,
            Err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission(_))
        ),
        "copied historical facts cannot turn a foreign instance into an actor wait"
    );

    let cancellation = WorthQueryCancellationSource::new();
    let cancelled_scope = WorthQueryRequestScope::new(
        std::time::Instant::now() + std::time::Duration::from_secs(60),
        cancellation.token(),
    );
    let cancelled_principal = authenticate_operator(runtime.installed_schema(), &cancelled_scope);
    cancellation.cancel();
    let cancelled = runtime
        .request(&cancelled_principal, &cancelled_scope)
        .on_branch(instance.branch())
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&9_217_010)
        .prepare_workflow_advance(&application, instance.clone());
    assert!(
        matches!(
            cancelled,
            Err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission(_))
        ),
        "cancelled admission must remain a typed denial"
    );
    let ordinary = run_instance(&application, instance.clone(), &[9_217_006]);
    assert!(ordinary.transitions().is_empty());
    assert_eq!(ordinary.attempted_steps(), 1);
    assert!(matches!(
        ordinary.stop(),
        WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingActor(_))
    ));

    change_advance_grant(&application, &instance, "active", 9_217_007);
    assert!(matches!(
        advance_instance(&application, instance, 9_217_008),
        Ok(WorkflowProgressOutcome::AwaitingCondition(_))
    ));
}

#[test]
fn completed_instance_is_not_relabelled_as_awaiting_an_actor() {
    let application = publish_workflow_on_first_program();
    let published = match publish_definition(
        &application,
        terminal_definition("done"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        9_218_000,
    )
    .expect("terminal workflow definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("terminal workflow definition did not publish: {other:?}"),
    };
    let instance = match start_instance(&application, published.definition().clone(), 9_218_001)
        .expect("terminal workflow instance prepares")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("terminal workflow instance did not start: {other:?}"),
    };
    change_advance_grant(&application, &instance, "revoked", 9_218_002);
    let waiting = advance_instance(&application, instance.clone(), 9_218_003);
    let Err(WorthQueryWorkflowAdvancePreparationDenial::AwaitingActor(required)) = waiting else {
        panic!("live terminal requires the next actor: {waiting:?}");
    };
    assert_eq!(required.node_path(), "done");
    assert_eq!(required.node_kind(), RequiredWorkflowActorNodeKind::Terminal);
    change_advance_grant(&application, &instance, "active", 9_218_004);
    assert!(matches!(
        advance_instance(&application, instance.clone(), 9_218_005),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    change_advance_grant(&application, &instance, "revoked", 9_218_006);
    assert!(matches!(
        advance_instance(&application, instance, 9_218_007),
        Err(WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission(_))
    ));
}

fn change_advance_grant(
    application: &super::bounded_dimension_model::host::BoundedDimensionWorkflowRuntime,
    instance: &worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    status: &str,
    key: u64,
) {
    change_grant(application, instance, "workflow-advance-grant", status, key);
}

fn change_grant(
    application: &super::bounded_dimension_model::host::BoundedDimensionWorkflowRuntime,
    instance: &worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    grant_identity: &str,
    status: &str,
    key: u64,
) {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let outcome = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowGrantStatusIntent {
            input: WorkflowGrantStatusInput {
                grant_identity: grant_identity.to_owned(),
                status: status.to_owned(),
            },
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(application.program_runtime())
        .expect("ordinary grant change prepares");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "ordinary grant change must publish: {outcome:?}"
    );
}
