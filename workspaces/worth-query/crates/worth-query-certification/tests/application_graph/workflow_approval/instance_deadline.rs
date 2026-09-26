//! A definition's total deadline bounds its instance's whole lineage on the
//! installed trusted clock. Once it elapses every step is refused, warm or
//! cold, and a step admitted in time is refused again at commit; a replay
//! still answers and cancellation still ends the instance. A successor keeps
//! the earlier of its source's deadline and its own.

use std::time::Duration;

use worth_query_execution::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestMutationDenial, WorthQueryWorkflowProposalPreparationDenial,
};

use super::super::bounded_dimension_model::{
    host::{publish_on_first_program_with_trusted_time, CertificationTrustedTime},
    workflow::{
        cancel_instance, migrate_instance, retain_workflow,
        reviewed_geometry_definition_with_deadline,
    },
};
use super::instance_cancellation::{approve, cancelled};
use super::instance_migration::{migration_denial, replace_definition, started};
use super::journey::approval_requirement;
use super::lineage_step_budget::step_denial;
use super::*;

const DEADLINE: Duration = Duration::from_secs(60);
const ELAPSED: WorthQueryApplicationAttemptDenialKind =
    WorthQueryApplicationAttemptDenialKind::WorkflowInstanceDeadlineElapsed;

/// A started instance of a definition declaring `DEADLINE`, on trusted time
/// the certification moves.
fn deadline_instance(
    key: u64,
) -> (
    CertificationTrustedTime,
    BoundedDimensionWorkflowRuntime,
    PublishedWorkflowDefinitionRef,
    PublishedWorkflowInstanceRef,
) {
    let time = CertificationTrustedTime::new();
    let application = retain_workflow(publish_on_first_program_with_trusted_time(time.clone()));
    let definition = match publish_definition(
        &application,
        reviewed_geometry_definition_with_deadline("applied", DEADLINE),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("the deadline definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the deadline definition did not publish: {other:?}"),
    };
    let instance = match start_instance(&application, definition.clone(), key + 1)
        .expect("the instance start prepares")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("the instance did not start: {other:?}"),
    };
    (time, application, definition, instance)
}

fn advance_denial(
    outcome: Result<WorkflowProgressOutcome, WorthQueryWorkflowAdvancePreparationDenial>,
) -> WorthQueryApplicationAttemptDenialKind {
    match outcome {
        Err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
            WorkflowTransitionPreparationDenial::Attempt(attempt),
        )) => attempt.kind(),
        Ok(WorkflowProgressOutcome::PreparationDenied(denial)) => denial.kind(),
        other => panic!("expected a typed step denial, got {other:?}"),
    }
}

fn warm_and_cold(application: &BoundedDimensionWorkflowRuntime, mut check: impl FnMut(&str)) {
    check("warm");
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    check("cold");
}

#[test]
fn a_step_is_refused_once_the_deadline_is_reached_but_its_replay_answers() {
    let (time, application, _, instance) = deadline_instance(93_000);
    time.advance(DEADLINE - Duration::from_millis(1));
    let proposal = match propose_instance(&application, instance.clone(), 93_002)
        .expect("a proposal a millisecond early prepares")
    {
        WorkflowProposalOutcome::Published(performed) => performed,
        other => panic!("a proposal before the deadline did not publish: {other:?}"),
    };
    assert!(!proposal.replayed());

    time.advance(Duration::from_millis(1));
    warm_and_cold(&application, |state| {
        match propose_instance(&application, instance.clone(), 93_002) {
            Ok(WorkflowProposalOutcome::Published(replay)) => {
                assert!(replay.replayed(), "the {state} retry replays");
                assert_eq!(replay.transition(), proposal.transition());
            }
            other => panic!("the {state} retry after the deadline must replay: {other:?}"),
        }
        assert_eq!(
            advance_denial(advance_instance(&application, instance.clone(), 93_003)),
            ELAPSED,
            "the {state} next step is refused at the deadline itself",
        );
    });
    let ended = cancelled(cancel_instance(&application, instance, 93_004));
    assert!(!ended.replayed(), "an overdue instance still cancels");
}

#[test]
fn a_fresh_proposal_after_the_deadline_names_it() {
    let (time, application, _, instance) = deadline_instance(93_100);
    time.advance(DEADLINE);
    warm_and_cold(&application, |state| {
        assert_eq!(
            step_denial(propose_instance(&application, instance.clone(), 93_102)),
            ELAPSED,
            "the {state} proposal is refused",
        );
    });
}

#[test]
fn an_approval_or_back_after_the_deadline_is_refused() {
    let (time, application, _, instance) = deadline_instance(93_200);
    let (proposal, required, _) = approval_requirement(&application, instance.clone(), 93_200);
    time.advance(DEADLINE);
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    warm_and_cold(&application, |state| {
        let back = runtime
            .request(&principal, &scope)
            .mutate(
                super::super::bounded_dimension_model::workflow::WorkflowAdvanceIntent {
                    input: super::super::bounded_dimension_model::workflow::WorkflowAdvanceInput {
                        part_identity: PART_IDENTITY.to_owned(),
                    },
                },
            )
            .without_source()
            .idempotency(&93_212_u64)
            .prepare_workflow_navigate_back(&application, instance.clone())
            .expect("Back receives fresh admission")
            .execute();
        assert!(
            matches!(
                back,
                Err(WorkflowProgressOutcome::PreparationDenied(ref denial))
                    if denial.kind() == ELAPSED
            ),
            "the {state} Back is refused: {back:?}",
        );
        assert_eq!(
            advance_denial(approve_instance(
                &application,
                instance.clone(),
                &required,
                &proposal,
                WorkflowApprovalDecision::Approve,
                93_210,
            )),
            ELAPSED,
            "the {state} approval is refused",
        );
    });
    let ended = cancelled(cancel_instance(&application, instance, 93_211));
    assert!(!ended.replayed());
}

/// Admits the approved operation in time, lets `lapse` act on trusted time,
/// then requests the operation: its commit refusal, or the currentness
/// refusal that stopped it before its handler.
fn overdue_operation(
    key: u64,
    lapse: impl FnOnce(&CertificationTrustedTime),
) -> Result<WorthQueryApplicationCommitDenialKind, WorthQueryApplicationAttemptDenialKind> {
    let (time, application, _, instance) = deadline_instance(key);
    let (proposal, required, _) = approval_requirement(&application, instance.clone(), key);
    approve(&application, &instance, &required, &proposal, key + 10);
    let operation = match advance_instance(&application, instance.clone(), key + 11) {
        Ok(WorkflowProgressOutcome::AwaitingOperation(operation)) => operation,
        other => panic!("expected an admitted operation, got {other:?}"),
    };
    lapse(&time);

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
        .idempotency(&(key + 12))
        .for_workflow_operation(&application, &operation)
        .expect("the request matches the requirement it was issued")
        .execute_in_program(application.program_runtime());
    assert_eq!(read_dimension(runtime, instance.branch()), SEED_DIMENSION);
    match effect {
        Ok(WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationCommitOutcome::Denied(denial),
        )) => Ok(denial.kind()),
        Err(WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness(denial)) => {
            Err(denial.kind())
        }
        other => panic!("an operation past trusted time never performs: {other:?}"),
    }
}

#[test]
fn an_operation_admitted_in_time_is_refused_at_commit_after_the_deadline() {
    assert_eq!(
        overdue_operation(93_300, |time| time.advance(DEADLINE)),
        Ok(WorthQueryApplicationCommitDenialKind::WorkflowSettlementDenied { kind: ELAPSED }),
    );
}

#[test]
fn a_successor_keeps_the_earlier_deadline_and_an_overdue_source_cannot_migrate() {
    let (time, application, definition, instance) = deadline_instance(93_400);
    approval_requirement(&application, instance.clone(), 93_400);
    time.advance(DEADLINE / 2);
    let target = replace_definition(
        &application,
        definition,
        reviewed_geometry_definition_with_deadline("done", DEADLINE),
        93_410,
    );
    let successor = started(migrate_instance(
        &application,
        instance,
        target.clone(),
        "propose",
        93_411,
    ))
    .instance()
    .clone();
    proposal::published_proposal(&application, successor.clone(), 93_412);

    time.advance(DEADLINE / 2);
    warm_and_cold(&application, |state| {
        assert_eq!(
            advance_denial(advance_instance(&application, successor.clone(), 93_413)),
            ELAPSED,
            "the {state} successor spends its source's deadline, not a fresh one",
        );
    });
    let next = replace_definition(
        &application,
        target,
        reviewed_geometry_definition_with_deadline("finished", DEADLINE),
        93_420,
    );
    assert_eq!(
        migration_denial(migrate_instance(
            &application,
            successor.clone(),
            next,
            "propose",
            93_421,
        )),
        ELAPSED,
        "an overdue instance cannot restart its deadline by migrating",
    );
    let ended = cancelled(cancel_instance(&application, successor, 93_422));
    assert!(!ended.replayed());
}

/// Trusted time that cannot be read admits nothing: authorization refuses a
/// new step, and the approval behind an operation admitted in time, before
/// any deadline is judged against an unknown instant.
#[test]
fn an_unreadable_trusted_clock_admits_no_step() {
    let (time, application, _, instance) = deadline_instance(93_500);
    time.set_unavailable(true);
    assert!(matches!(
        propose_instance(&application, instance.clone(), 93_502),
        Err(WorthQueryWorkflowProposalPreparationDenial::RequestAdmission(
            WorthQueryApplicationRequestMutationDenial::Authorization(ref denial)
        )) if denial.kind() == WorthQueryOperationAuthorizationDenialKind::TrustedTimeUnavailable
    ));
    time.set_unavailable(false);
    match propose_instance(&application, instance, 93_503) {
        Ok(WorkflowProposalOutcome::Published(_)) => {}
        other => panic!("a readable clock admits the step again: {other:?}"),
    }
    assert_eq!(
        overdue_operation(93_600, |time| time.set_unavailable(true)),
        Err(WorthQueryApplicationAttemptDenialKind::WorkflowApprovalAuthorityDenied),
    );
}
