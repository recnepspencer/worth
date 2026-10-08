//! Real installed publication attempts: work belongs to an attempt, not its receipt.
use super::amendment;
use super::product_workflow_support::{self, principal, read_input, ExampleApplication};
use product_workflow_support::application_entry::AmendTemporalDenial;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt, WorthQueryApplicationRequestMutationDenial as Denial,
    },
    primary_graph::{WorthQueryInvariantProjectionWork, WorthQueryMutationHandlerWork},
};

fn captured(work: WorthQueryMutationHandlerWork) -> WorthQueryInvariantProjectionWork {
    let WorthQueryMutationHandlerWork::Captured(capture) = work else {
        panic!("the installed decision must produce an execution-owned capture")
    };
    assert!(capture.handler_contacted());
    let projection = capture.projection_work();
    assert!(projection.provider_work_units() > 0);
    assert!(projection.field_reads() > 0);
    projection
}

#[test]
fn mutation_attempt_report_distinguishes_preflight_domain_success_and_early_replay() {
    let mut app = ExampleApplication::publish("blocked");
    let scope = product_workflow_support::adapters::request_scope();
    let principal = principal(&app, &scope);
    let branch = app.runtime.current_world();
    let before = read_input(&app, branch, &principal, &scope);
    let request = app.runtime.request(&principal, &scope);

    let preflight = request
        .mutate(amendment("next", 2))
        .without_source()
        .idempotency(&0x81_u64)
        .execute_report(
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert_eq!(
        *preflight.decision_work(),
        WorthQueryMutationHandlerWork::NotStarted
    );
    assert!(matches!(
        preflight.into_outcome(),
        Err(Denial::ApplicationProgramRequired)
    ));

    let domain = request
        .mutate(amendment("refused", 1))
        .without_source()
        .idempotency(&0x82_u64)
        .execute_in_program_report(
            &app.runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    let (outcome, work) = domain.into_parts();
    captured(work);
    assert!(matches!(
        outcome.unwrap(),
        Outcome::DomainDenied(AmendTemporalDenial::RevisionMustAdvance)
    ));
    assert_eq!(read_input(&app, branch, &principal, &scope), before);

    let key = 0x83_u64;
    let fresh = request
        .mutate(amendment("next", 2))
        .without_source()
        .idempotency(&key)
        .execute_in_program_report(
            &app.runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    let (outcome, work) = fresh.into_parts();
    captured(work);
    let outcome = outcome.unwrap();
    assert!(matches!(outcome, Outcome::Committed { .. }));
    assert_eq!(outcome.result().unwrap().revision, 2);
    let receipt = outcome.receipt().unwrap().clone();
    let after = read_input(&app, branch, &principal, &scope);
    assert_ne!(after, before);

    let replay = request
        .mutate(amendment("next", 2))
        .without_source()
        .idempotency(&key)
        .execute_in_program_report(
            &app.runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert_eq!(
        *replay.decision_work(),
        WorthQueryMutationHandlerWork::NotStarted
    );
    let replay = replay.into_outcome().unwrap();
    assert!(matches!(replay, Outcome::AlreadyCommitted(_)));
    assert!(replay.result().is_none());
    assert_eq!(
        replay.receipt().unwrap().commit_reference(),
        receipt.commit_reference()
    );
    assert_eq!(read_input(&app, branch, &principal, &scope), after);
    app.runtime.close_conditional_runtime().unwrap();
}

#[test]
fn mutation_attempt_report_keeps_preparation_capture_on_late_duplicate() {
    let mut app = ExampleApplication::publish("blocked");
    let scope = product_workflow_support::adapters::request_scope();
    let principal = principal(&app, &scope);
    let request = app.runtime.request(&principal, &scope);
    let key = 0x84_u64;
    let mut first = request
        .mutate(amendment("prepared", 2))
        .without_source()
        .idempotency(&key);
    let mut second = request
        .mutate(amendment("prepared", 2))
        .without_source()
        .idempotency(&key);
    let (first, first_work) = first
        .prepare_in_program_report(
            &app.runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .into_parts();
    let (second, second_work) = second
        .prepare_in_program_report(
            &app.runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .into_parts();
    captured(first_work);
    captured(second_work);
    let Preparation::Prepared(first) = first.unwrap() else {
        panic!("first candidate must prepare")
    };
    let Preparation::Prepared(second) = second.unwrap() else {
        panic!("second candidate must prepare")
    };
    let (fresh, committed_work) = first
        .commit_report(
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .into_parts();
    assert_eq!(committed_work, first_work);
    assert!(matches!(fresh, Outcome::Committed { .. }));
    let (duplicate, duplicate_work) = second
        .commit_report(
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .into_parts();
    assert_eq!(duplicate_work, second_work);
    assert!(matches!(duplicate, Outcome::AlreadyCommitted(_)));
    assert!(duplicate.result().is_none());
    assert_eq!(
        duplicate.receipt().unwrap().commit_reference(),
        fresh.receipt().unwrap().commit_reference()
    );
    app.runtime.close_conditional_runtime().unwrap();
}
