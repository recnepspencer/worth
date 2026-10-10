use super::program_fixture::admitted_program;
use super::{
    authenticated_principal, idempotency, installed_authorization_world, live_scope,
    resolved_account,
};
use crate::domain_computation::primary_graph::tests::fixture::MultiTouchOperation;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitTerminalKind, WorthQueryApplicationIdempotencyResolution,
};
use crate::facade::runtime::ExecutionAllocationPolicy;

#[path = "idempotency_behavior/equivalent_retry_recovers_original_receipt_while_intent_drift_is.rs"]
mod equivalent_retry_recovers_original_receipt_while_intent_drift_is;

#[test]
fn equivalent_retry_prepared_at_the_original_product_recovers_exact_owner_evidence() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let selected_commit = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .product()
                .selected_commit()
                .clone()
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(&phase, &world, &principal, &account, &request, "committed")
        })
        .expect("the fixture read owns its installed host call");
    let stale_retry = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(&phase, &world, &principal, &account, &request, "committed")
        })
        .expect("the fixture read owns its installed host call");
    let binding = idempotency(93, 93);

    let WorthQueryApplicationCommitOutcome::Committed(committed) =
        world.application.compare_and_commit_application(
            first,
            binding,
            ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the first application must publish its exact owner evidence");
    };
    let current_commit = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .product()
                .selected_commit()
                .clone()
        })
        .expect("the fixture read owns its installed host call");

    let outcome = world.application.compare_and_commit_application(
        stale_retry,
        binding,
        ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) = outcome else {
        panic!("an equivalent retry must recover the exact completed owner evidence: {outcome:?}");
    };
    assert!(recovered.is_same_authoritative_commit(&committed));
    assert_eq!(
        recovered.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Recovered
    );
    assert_ne!(selected_commit, current_commit);
}

#[test]
fn same_caller_intent_cannot_cross_installed_operation_identity() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(&phase, &world, &principal, &account, &request, "committed")
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            first,
            idempotency(10, 10),
            ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));

    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "committed", &request)
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(MultiTouchOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let resolution = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .resolve_admitted_application_idempotency(&phase, &admission, idempotency(10, 10))
                .unwrap()
                .into_resolution()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        resolution,
        WorthQueryApplicationIdempotencyResolution::IntentDrift
    );
}

#[test]
fn same_operation_intent_cannot_cross_an_admitted_scope() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let first_account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(
                &phase,
                &world,
                &principal,
                &first_account,
                &request,
                "first-scope",
            )
        })
        .expect("the fixture read owns its installed host call");

    assert!(matches!(
        world.application.compare_and_commit_application(
            first,
            idempotency(11, 11),
            ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let second_account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "unrelated", &request)
        })
        .expect("the fixture read owns its installed host call");
    let second = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(
                &phase,
                &world,
                &principal,
                &second_account,
                &request,
                "second-scope",
            )
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            second,
            idempotency(11, 11),
            ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("reusing one operation intent across scopes must deny");
    };
    assert_eq!(
        denial.kind(),
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
    );
}

#[path = "idempotency_behavior/lifecycle_replay_resolvers_have_no_replay_for_an_ordinary_admiss.rs"]
mod lifecycle_replay_resolvers_have_no_replay_for_an_ordinary_admiss;

#[path = "idempotency_behavior/history_retirement.rs"]
mod history_retirement;
