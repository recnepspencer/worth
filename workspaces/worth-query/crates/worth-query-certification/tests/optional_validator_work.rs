//! A caller work budget changes attempt eligibility, never input/source/key meaning.
use super::{amendment, product_workflow_support};
use product_workflow_support::{principal, read_input, ExampleApplication};
use std::num::NonZeroU64;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
    },
    primary_graph::{
        WorthQueryApplicationUncommitted as Uncommitted,
        WorthQueryInvariantExecutionDenialKind as Kind,
    },
};

#[test]
fn optional_validator_work_refuses_without_effect_then_same_identity_commits() {
    let mut app = ExampleApplication::publish("blocked");
    let scope = product_workflow_support::adapters::request_scope();
    assert_eq!(scope.candidate_validator_work_budget(), None);
    let bounded = scope
        .clone()
        .with_candidate_validator_work_budget(NonZeroU64::new(1).unwrap());
    let principal = principal(&app, &scope);
    let branch = app.runtime.current_world();
    let before = read_input(&app, branch, &principal, &scope);
    let key = 0x91_u64;
    let denied = app
        .runtime
        .request(&principal, &bounded)
        .mutate(amendment("budget-retry", 2))
        .without_source()
        .idempotency(&key)
        .execute_in_program_report(&app.runtime)
        .into_outcome()
        .unwrap();
    let Outcome::Commit(Uncommitted::Denied(denial)) = denied else {
        panic!("expected actual validator refusal: {denied:?}");
    };
    assert!(
        matches!(denial.invariant_execution_failure().unwrap().kind(),
        Kind::CandidateValidatorWorkExceeded { maximum_work: 1, required_work } if required_work > 1)
    );
    assert_eq!(read_input(&app, branch, &principal, &scope), before);
    let fresh = app
        .runtime
        .request(&principal, &scope)
        .mutate(amendment("budget-retry", 2))
        .without_source()
        .idempotency(&key)
        .execute_in_program(&app.runtime)
        .unwrap();
    assert!(
        matches!(fresh, Outcome::Committed { .. }),
        "same input/key must commit: {fresh:?}"
    );
    let receipt = fresh.receipt().unwrap().clone();
    let replay = app
        .runtime
        .request(&principal, &bounded)
        .mutate(amendment("budget-retry", 2))
        .without_source()
        .idempotency(&key)
        .execute_in_program(&app.runtime)
        .unwrap();
    assert!(
        matches!(replay, Outcome::AlreadyCommitted(_)),
        "work policy must not change replay identity: {replay:?}"
    );
    assert_eq!(
        replay.receipt().unwrap().commit_reference(),
        receipt.commit_reference()
    );
    app.runtime.close_conditional_runtime().unwrap();
}
