//! Phase 4 retires the zero-custody refusal: actual recovered effects reach service.
use super::pending_support::{balance, pending, scope, set};
use super::*;
use worth_query_host::facade::primary_graph::WorldRecoveredContinuationOutcome as Continuation;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome as Mutation,
    primary_graph::{
        ProductUnpublishedCause, ProductUnpublishedNextAction, WorthQueryApplicationUncommitted,
    },
};
#[test]
fn recovered_original_custody_reaches_service_and_continues_without_reexecution() {
    journey(true, false);
}
#[test]
fn recovered_original_custody_reaches_service_and_release_stays_released() {
    journey(false, false);
}
#[test]
fn continued_recovered_custody_closes_cooperatively_and_resumes_after_reopen() {
    journey(true, true);
}
fn journey(continue_original: bool, close_prepared: bool) {
    use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
    let location = fixture::location();
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = fixture::request(
        location.clone(),
        build,
        initialized.clone(),
        contacts.clone(),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("actual writer: {refused:?}"));
    let (_, live) = scope();
    assert!(matches!(
        set(home.runtime(), &live, 0x9920, 8).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { result: 8, .. }
    ));
    home.runtime()
        .runtime()
        .world_operation_control_for_test()
        .panic_before_product_compare_once_for_test();
    let pending = pending(set(home.runtime(), &live, 0x9921, 9));
    let recovery = pending
        .try_into_recovery()
        .unwrap_or_else(|pending| panic!("same actual post-Native owner: {pending:?}"));
    let effects = recovery.inspect().unwrap();
    let original_identity = recovery.record_handle().identity().clone();
    let original_attempt = effects.attempt_identity().clone();
    let original_meaning = serde_json::to_value(effects.native_semantic_result().unwrap()).unwrap();
    let predecessor = effects.expected_product().selected_commit().clone();
    assert!(
        effects.owner_effect_count() > 0,
        "the actual control unwinds after the performed Native effect"
    );
    assert_eq!(balance(home.runtime(), &live), 8);
    assert_eq!(contacts.load(Ordering::SeqCst), 2);

    drop(effects);
    drop(recovery);
    drop(home);
    let open = || {
        let request = fixture::request_with_initializer(
            location.clone(),
            build,
            contacts.clone(),
            |_, _, _| panic!("existing Home cannot initialize"),
        );
        with_bootstrap_advancement_from_request(execution, |phase| {
            request.open_in_advancement(&phase)
        })
        .unwrap()
        .unwrap_or_else(|refused| panic!("typed original custody must become service: {refused:?}"))
    };
    let mut home = open();
    assert_eq!(balance(home.runtime(), &live), 8);
    let effects = match set(home.runtime(), &live, 0x9921, 9).unwrap() {
        Mutation::Commit(WorthQueryApplicationUncommitted::ProductUnpublished(effects)) => effects,
        other => panic!("original retry must expose original recovered custody: {other:?}"),
    };
    assert_eq!(
        effects.cause(),
        ProductUnpublishedCause::RecoveredWithoutProductMovement
    );
    assert_eq!(effects.attempt_identity(), &original_attempt);
    assert_eq!(effects.expected_product().selected_commit(), &predecessor);
    assert_eq!(
        serde_json::to_value(effects.native_semantic_result().unwrap()).unwrap(),
        original_meaning
    );
    assert!(effects
        .next_actions()
        .contains(&ProductUnpublishedNextAction::ContinueOriginalPublication));
    assert!(effects
        .next_actions()
        .contains(&ProductUnpublishedNextAction::ReleaseObligations));
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    assert!(matches!(
        set(home.runtime(), &live, 0x9921, 10).unwrap(),
        Mutation::IdempotencyIntentDrift
    ));
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    let mut recovery = effects.into_recovery();
    assert_eq!(recovery.record_handle().identity(), &original_identity);
    if close_prepared {
        let observation = home
            .runtime()
            .runtime()
            .world_operation_control_for_test()
            .observe_recovered_continued_for_test(|| {
                panic!("actual Continued acknowledged; occurrence not submitted")
            });
        let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            recovery.continue_original_in_request(&execution)
        }));
        assert!(
            interrupted.is_err(),
            "actual control interrupts the acknowledged disposition"
        );
        drop(observation);
        drop(recovery);
        home.close(&execution).unwrap_or_else(|refused| {
            panic!("prepared continuation must retire through actual close: {refused:?}")
        });
        home = open();
        let effects = match set(home.runtime(), &live, 0x9921, 9).unwrap() {
            Mutation::Commit(WorthQueryApplicationUncommitted::ProductUnpublished(effects)) => {
                effects
            }
            other => panic!("same original Continued after close: {other:?}"),
        };
        assert_eq!(effects.attempt_identity(), &original_attempt);
        assert!(!effects
            .next_actions()
            .contains(&ProductUnpublishedNextAction::ReleaseObligations));
        assert_eq!(
            serde_json::to_value(effects.native_semantic_result().unwrap()).unwrap(),
            original_meaning
        );
        recovery = effects.into_recovery();
        assert_eq!(contacts.load(Ordering::SeqCst), 2);
    }
    if continue_original {
        loop {
            match recovery.continue_original_in_request(&execution).unwrap() {
                Continuation::Pending(custody) => drop(custody),
                Continuation::Published { .. } => break,
                other => panic!("actual original continuation: {other:?}"),
            }
        }
        assert_eq!(balance(home.runtime(), &live), 9);
    } else {
        let mut retained = recovery.clone();
        loop {
            match home.runtime().runtime().release_product_publication_recovery(retained, 0) {
                Ok(_) => break,
                Err(worth_query_host::facade::primary_graph::WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(failure))
                    if failure.denial() == worth_query_host::facade::primary_graph::WorthQueryProductUnpublishedRecoveryReleaseDenial::RecoveredPending => {
                    let next = failure.into_recovery();
                    // A pending original Released cannot be replaced by Continued.
                    match next.continue_original_in_request(&execution).unwrap() {
                        Continuation::Refused { cause: worth_query_host::facade::primary_graph::WorldRecoveredContinuationDenial::Admission(worth_query_host::facade::primary_graph::RuntimeWorldRecoveryDenial::ReleaseAlreadyStarted), custody } => drop(custody),
                        other => panic!("pending release must retain its single chosen disposition: {other:?}"),
                    }
                    retained = next;
                }
                Err(other) => panic!("actual durable release: {other:?}"),
            }
        }
        assert_eq!(balance(home.runtime(), &live), 8);
    }
    drop(recovery);
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    drop(home);
    let home = open();
    assert_eq!(
        balance(home.runtime(), &live),
        if continue_original { 9 } else { 8 }
    );
    assert!(home
        .runtime()
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(32).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    if continue_original {
        match set(home.runtime(), &live, 0x9921, 9).unwrap() {
            Mutation::RecoveredCommitted(receipt) => assert_eq!(
                serde_json::to_value(receipt.semantic_result().semantic()).unwrap(),
                original_meaning
            ),
            other => {
                panic!("continued occurrence must remain original and published once: {other:?}")
            }
        }
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("readmitted disposition close: {refused:?}"));
}
