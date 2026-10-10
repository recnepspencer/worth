use super::{
    admitted_program_with_emit, authenticated_principal, idempotency, resolved_account,
    AccountStatus, TouchAccountOperation,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, AccountActivityEffect,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitTerminalKind, WorthQueryApplicationHistoricalRead,
    WorthQueryApplicationQueryAdmissionDenialKind,
};
use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;

#[test]
fn typed_emission_is_published_with_the_exact_provider_commit() {
    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let _live = world
        .application
        .primary_provider
        .observe_application_commit_causality(selected.product());
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_with_emit(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "emitted",
                Some("account-activity"),
            )
        })
        .expect("the fixture read owns its installed host call");

    let WorthQueryApplicationCommitOutcome::Committed(receipt) =
        world.application.compare_and_commit_application(
            program,
            idempotency(31, 31),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("typed emission attempt must commit");
    };
    assert_eq!(receipt.emitted_effect_count(), 1);
    let emissions = world
        .application
        .primary_provider
        .committed_application_emissions(receipt.committed_product_publication());
    assert_eq!(emissions.len(), 1);
    assert_eq!(emissions[0].effect(), "AccountActivityEffect");
    assert_eq!(
        emissions[0].payload::<String>().map(String::as_str),
        Some("account-activity")
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .retained_application_emission_bytes(),
        emissions[0].retained_bytes(),
        "the live source must account for the exact admitted payload bytes"
    );
}

#[test]
fn response_loss_recovers_emit_receipt_without_duplicate_publication() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_with_emit(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "published",
                Some("once"),
            )
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_with_emit(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "published",
                Some("once"),
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.lose_next_commit_response();
    let WorthQueryApplicationCommitOutcome::Committed(original) =
        world.application.compare_and_commit_application(
            first,
            idempotency(33, 33),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("idempotency must recover the response-lost emit commit");
    };
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) =
        world.application.compare_and_commit_application(
            retry,
            idempotency(33, 33),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("retry must recover the exact emit commit");
    };
    assert!(recovered.is_same_authoritative_commit(&original));
    assert_eq!(
        original.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Executed
    );
    assert_eq!(
        recovered.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Recovered
    );
    assert_eq!(recovered.emitted_effect_count(), 1);
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        1
    );
}

#[path = "emitted_effects/external_receipt_clones_do_not_pin_an_evicted_historical_basis.rs"]
mod external_receipt_clones_do_not_pin_an_evicted_historical_basis;

#[test]
fn in_window_receipt_admits_its_exact_historical_basis() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_with_emit(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "historical",
                None,
            )
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationCommitOutcome::Committed(receipt) =
        world.application.compare_and_commit_application(
            first,
            idempotency(41, 41),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("historical transaction must commit");
    };
    let exact_version = receipt.commit_reference().version_id;
    let external_clone = receipt.clone();

    let current_account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "historical", &request)
        })
        .expect("the fixture read owns its installed host call");
    let advanced = world
        .application
        .with_host_advancement(|phase| {
            admitted_program_with_emit(
                &phase,
                &world,
                &principal,
                &current_account,
                &request,
                "advanced",
                None,
            )
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationCommitOutcome::Committed(advanced) =
        world.application.compare_and_commit_application(
            advanced,
            idempotency(42, 42),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("advancing transaction must commit");
    };
    assert_ne!(advanced.commit_reference().version_id, exact_version);

    let historical_read =
        WorthQueryApplicationHistoricalRead::at_application_commit(&external_clone);
    drop(external_clone);
    drop(receipt);
    let basis = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .admit_application_historical_basis(
                    &host_custody.borrowed_request_for_test(),
                    historical_read,
                    &request,
                )
                .expect("the bounded owner must retain an in-window receipt basis")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(basis.version_id(), exact_version);
    assert!(basis.release().released());
}

#[path = "emitted_effects/cumulative_variable_width_payloads_are_denied_before_provider_co.rs"]
mod cumulative_variable_width_payloads_are_denied_before_provider_co;

#[path = "emitted_effects/pretransaction_emit_refusal.rs"]
mod pretransaction_emit_refusal;
