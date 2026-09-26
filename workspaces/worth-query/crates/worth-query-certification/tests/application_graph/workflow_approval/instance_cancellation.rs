//! Explicit cancellation ends a live instance where it stands. It is not
//! rollback: a performed effect remains and is reported, a step admitted
//! before the cancellation goes stale before its effect, and a cancellation
//! prepared before a step settles goes stale in turn.

use worth_query_host::facade::application_entry::{
    PerformedWorkflowInstanceCancellation, WorkflowInstanceBindingDenial,
    WorkflowInstanceCancellationOutcome, WorkflowInstancePreparationDenial,
    WorthQueryWorkflowInstanceStartPreparationDenial,
};

use super::super::bounded_dimension_model::{
    programs::DimensionProgramP1,
    workflow::{
        cancel_instance, cancel_on_second, prepare_cancellation, prepare_second_program_adoption,
        publish_adoption, support_workflow_program,
    },
};
use super::instance_migration::perform_approved_effect;
use super::*;

#[test]
fn a_cancel_before_the_effect_leaves_the_admitted_step_stale() {
    let (application, _, instance, proposal, required, _) = approval_journey("applied", 88_000);
    approve(&application, &instance, &required, &proposal, 88_010);
    let operation = match advance_instance(&application, instance.clone(), 88_011) {
        Ok(WorkflowProgressOutcome::AwaitingOperation(operation)) => operation,
        other => panic!("expected an admitted operation, got {other:?}"),
    };

    let done = cancelled(cancel_instance(&application, instance.clone(), 88_012));
    assert!(!done.replayed());
    assert_eq!(done.instance(), &instance);
    assert!(done.performed_node_paths().is_empty());
    let replay = cancelled(cancel_instance(&application, instance.clone(), 88_012));
    assert!(replay.replayed(), "the same key replays exactly");
    assert_eq!(
        replay.receipt().outcome_identity(),
        done.receipt().outcome_identity()
    );
    assert_eq!(
        cancellation_denial(cancel_instance(&application, instance.clone(), 88_013)),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCancelled,
        "another request for a cancelled instance is refused as cancelled",
    );

    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let effect = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetPartDimensionIntent {
            input: super::super::bounded_dimension_model::schema::SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .without_source()
        .idempotency(&88_014)
        .for_workflow_operation(&application, &operation)
        .expect("the request still matches the requirement it was issued")
        .execute_in_program(application.program_runtime());
    assert!(
        !matches!(
            effect,
            Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
        ),
        "a step admitted before the cancellation never performs: {effect:?}"
    );
    assert_eq!(
        read_dimension(application.runtime(), instance.branch()),
        SEED_DIMENSION
    );
    match advance_instance(&application, instance, 88_015) {
        Err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
            WorkflowTransitionPreparationDenial::Attempt(attempt),
        )) => assert_eq!(
            attempt.kind(),
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCancelled
        ),
        other => panic!("a cancelled instance names its cancellation: {other:?}"),
    }
}

#[test]
fn a_cancel_after_the_effect_reports_it_and_leaves_it_performed() {
    let (application, _, instance, proposal, required, _) = approval_journey("applied", 88_100);
    approve(&application, &instance, &required, &proposal, 88_110);
    perform_approved_effect(&application, &instance, 88_111);

    let done = cancelled(cancel_instance(&application, instance.clone(), 88_113));
    assert_eq!(done.performed_node_paths(), ["apply".to_owned()]);
    let replay = cancelled(cancel_instance(&application, instance.clone(), 88_113));
    assert!(replay.replayed());
    assert_eq!(
        replay.performed_node_paths(),
        ["apply".to_owned()],
        "a replay reports the same performed effect from settled history",
    );
    assert_eq!(read_dimension(application.runtime(), instance.branch()), 8);
}

#[test]
fn a_cancel_prepared_before_the_effect_goes_stale_when_the_effect_lands_first() {
    let (application, _, instance, proposal, required, _) = approval_journey("applied", 88_200);
    approve(&application, &instance, &required, &proposal, 88_210);
    let early = prepare_cancellation(&application, instance.clone(), 88_211)
        .expect("the cancellation prepares before the effect");
    perform_approved_effect(&application, &instance, 88_212);
    match early() {
        WorkflowInstanceCancellationOutcome::Application(outcome) => assert!(
            !matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
            "{outcome:?}"
        ),
        other => panic!("a cancellation read before the effect is stale: {other:?}"),
    }
    // The stale attempt claimed no key: the same key now cancels afresh and
    // reports the effect that won the race.
    let done = cancelled(cancel_instance(&application, instance.clone(), 88_211));
    assert!(!done.replayed());
    assert_eq!(done.performed_node_paths(), ["apply".to_owned()]);
    assert_eq!(read_dimension(application.runtime(), instance.branch()), 8);
}

#[test]
fn a_completed_instance_has_nothing_left_to_cancel_and_a_sibling_runs_on() {
    let (application, definition, instance, proposal, required, _) =
        approval_journey("applied", 88_300);
    let sibling = match start_instance(&application, definition, 88_320)
        .expect("a sibling instance prepares")
    {
        WorkflowInstanceStartOutcome::Started(started) => started.instance().clone(),
        other => panic!("expected a sibling instance, got {other:?}"),
    };
    // Cancelling one instance leaves its sibling free to run to completion.
    cancelled(cancel_instance(&application, sibling, 88_321));
    approve(&application, &instance, &required, &proposal, 88_310);
    perform_approved_effect(&application, &instance, 88_311);
    match advance_instance(&application, instance.clone(), 88_313) {
        Ok(WorkflowProgressOutcome::Completed(performed)) => assert!(performed.terminal()),
        other => panic!("the instance completes at its terminal: {other:?}"),
    }
    assert_eq!(
        cancellation_denial(cancel_instance(&application, instance, 88_314)),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCompleted,
    );
}

#[test]
fn a_cancellation_replays_exactly_after_its_branch_adopts_a_new_program() {
    let (mut application, _, instance, _, _, _) = approval_journey("approved", 88_400);
    let done = cancelled(cancel_instance(&application, instance.clone(), 88_410));
    support_workflow_program::<DimensionProgramP1>(&mut application);
    let main = application.current_world();
    publish_adoption(prepare_second_program_adoption(
        &application,
        main,
        Some(&|inventory| inventory.carry_compatible().unwrap()),
    ));

    match cancel_instance(&application, instance.clone(), 88_410) {
        Err(WorthQueryWorkflowInstanceStartPreparationDenial::InstancePreparation(
            WorkflowInstancePreparationDenial::Binding(
                WorkflowInstanceBindingDenial::ProgramRevisionChanged,
            ),
        )) => {}
        other => panic!("the retired program's vocabulary is refused: {other:?}"),
    }
    let replay = cancelled(cancel_on_second(&application, instance.clone(), 88_410));
    assert!(
        replay.replayed(),
        "the key replays under the adopted program"
    );
    assert_eq!(
        replay.receipt().outcome_identity(),
        done.receipt().outcome_identity()
    );
    assert_eq!(
        cancellation_denial(cancel_on_second(&application, instance, 88_411)),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCancelled,
    );
}

fn approve(
    application: &BoundedDimensionWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowApproval,
    proposal: &PublishedWorkflowProposalRef,
    key: u64,
) {
    match approve_instance(
        application,
        instance.clone(),
        required,
        proposal,
        WorkflowApprovalDecision::Approve,
        key,
    ) {
        Ok(WorkflowProgressOutcome::Completed(_)) => {}
        other => panic!("expected approval to complete, got {other:?}"),
    }
}

fn cancelled(
    outcome: Result<
        WorkflowInstanceCancellationOutcome,
        WorthQueryWorkflowInstanceStartPreparationDenial,
    >,
) -> PerformedWorkflowInstanceCancellation {
    match outcome.expect("the cancellation prepares") {
        WorkflowInstanceCancellationOutcome::Cancelled(performed) => performed,
        other => panic!("the cancellation did not commit: {other:?}"),
    }
}

fn cancellation_denial(
    outcome: Result<
        WorkflowInstanceCancellationOutcome,
        WorthQueryWorkflowInstanceStartPreparationDenial,
    >,
) -> WorthQueryApplicationAttemptDenialKind {
    match outcome {
        Err(WorthQueryWorkflowInstanceStartPreparationDenial::InstancePreparation(
            WorkflowInstancePreparationDenial::Attempt(attempt),
        )) => attempt.kind(),
        other => panic!("expected a typed cancellation denial, got {other:?}"),
    }
}
