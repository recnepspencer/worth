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

#[test]
fn typed_emission_is_published_with_the_exact_provider_commit() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let _live = world
        .application
        .primary_provider
        .observe_application_commit_causality(selected.product());
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program = admitted_program_with_emit(
        &world,
        &principal,
        &account,
        &request,
        "emitted",
        Some("account-activity"),
    );

    let WorthQueryApplicationCommitOutcome::Committed(receipt) = world
        .application
        .compare_and_commit_application(program, idempotency(31, 31))
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
fn rejection_before_transaction_publishes_no_emit_causality() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program = admitted_program_with_emit(
        &world,
        &principal,
        &account,
        &request,
        "rejected",
        Some("must-not-publish"),
    );

    world.faults.reject_next_commit_before_transaction();
    let outcome = world
        .application
        .compare_and_commit_application(program, idempotency(32, 32));
    assert!(
        !matches!(
            outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
                | WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
        ),
        "rejected transaction claimed commit: {outcome:?}"
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        0
    );
}

#[test]
fn response_loss_recovers_emit_receipt_without_duplicate_publication() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let first = admitted_program_with_emit(
        &world,
        &principal,
        &account,
        &request,
        "published",
        Some("once"),
    );
    let retry = admitted_program_with_emit(
        &world,
        &principal,
        &account,
        &request,
        "published",
        Some("once"),
    );

    world.faults.lose_next_commit_response();
    let WorthQueryApplicationCommitOutcome::Committed(original) = world
        .application
        .compare_and_commit_application(first, idempotency(33, 33))
    else {
        panic!("idempotency must recover the response-lost emit commit");
    };
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) = world
        .application
        .compare_and_commit_application(retry, idempotency(33, 33))
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

#[test]
fn external_receipt_clones_do_not_pin_an_evicted_historical_basis() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let _live = world
        .application
        .primary_provider
        .observe_application_commit_causality(selected.product());
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let first = admitted_program_with_emit(
        &world,
        &principal,
        &account,
        &request,
        "first-emitted",
        Some("recover-after-eviction"),
    );
    let retry = admitted_program_with_emit(
        &world,
        &principal,
        &account,
        &request,
        "first-emitted",
        Some("recover-after-eviction"),
    );
    let WorthQueryApplicationCommitOutcome::Committed(original) = world
        .application
        .compare_and_commit_application(first, idempotency(40, 40))
    else {
        panic!("first emitted transaction must commit");
    };
    let external_original_clone = original.clone();

    let mut current_status = "first-emitted".to_string();
    for ordinal in 0..64_u8 {
        let current_account = resolved_account(&world, &current_status, &request);
        let replacement = format!("eviction-filler-{ordinal}");
        let program = admitted_program_with_emit(
            &world,
            &principal,
            &current_account,
            &request,
            &replacement,
            None,
        );
        assert!(matches!(
            world
                .application
                .compare_and_commit_application(program, idempotency(ordinal + 64, ordinal + 64)),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
        current_status = replacement;
    }
    assert!(
        world
            .application
            .primary_provider
            .committed_application_emissions(original.committed_product_publication())
            .is_empty(),
        "the first batch must be absent from the bounded live source"
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .retained_application_emission_bytes(),
        0,
        "evicting the only nonempty batch must release its accounted payload bytes"
    );

    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) = world
        .application
        .compare_and_commit_application(retry, idempotency(40, 40))
    else {
        panic!("provider idempotency must recover after live-source eviction");
    };
    assert!(recovered.is_same_authoritative_commit(&original));
    assert_eq!(
        recovered.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Recovered
    );
    assert_eq!(recovered.emitted_effect_count(), 1);
    let external_recovered_clone = recovered.clone();
    assert_eq!(
        world
            .application
            .primary_provider
            .retained_receipt_basis_count(),
        64,
        "completed evidence must not grow the bounded receipt-basis owner"
    );
    let expired_read =
        WorthQueryApplicationHistoricalRead::at_application_commit(&external_recovered_clone);
    let denial = world
        .application
        .admit_application_historical_basis(expired_read, &request)
        .err()
        .expect("external receipt clones must not pin an evicted historical basis");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable
    );
    assert!(external_original_clone.is_same_authoritative_commit(&original));
    assert!(external_recovered_clone.is_same_authoritative_commit(&recovered));
}

#[test]
fn in_window_receipt_admits_its_exact_historical_basis() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let first =
        admitted_program_with_emit(&world, &principal, &account, &request, "historical", None);
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = world
        .application
        .compare_and_commit_application(first, idempotency(41, 41))
    else {
        panic!("historical transaction must commit");
    };
    let exact_version = receipt.commit_reference().version_id;
    let external_clone = receipt.clone();

    let current_account = resolved_account(&world, "historical", &request);
    let advanced = admitted_program_with_emit(
        &world,
        &principal,
        &current_account,
        &request,
        "advanced",
        None,
    );
    let WorthQueryApplicationCommitOutcome::Committed(advanced) = world
        .application
        .compare_and_commit_application(advanced, idempotency(42, 42))
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
        .admit_application_historical_basis(historical_read, &request)
        .expect("the bounded owner must retain an in-window receipt basis");
    assert_eq!(basis.version_id(), exact_version);
    assert!(basis.release().released());
}

#[test]
fn cumulative_variable_width_payloads_are_denied_before_provider_commit() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, projected| {
            reader
                .require_decision_field(projected, AccountStatus::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies()
        .unwrap()
        .begin_effect_program();
    let first = String::with_capacity(140_000);
    let second = String::with_capacity(140_000);

    effects
        .emit(AccountActivityEffect::reference(), first)
        .expect("one variable-width payload fits the installed byte envelope");
    let denial = effects
        .emit(AccountActivityEffect::reference(), second)
        .unwrap_err();

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::RetainedEffectBytesExceeded
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        0,
        "payload admission must fail before provider mutation or publication"
    );
}
