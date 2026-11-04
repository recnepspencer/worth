//! Continue or release the same recovered effect before the terminal reopen.
use super::*;

struct ContinuedBoundaryReached;
pub(super) fn apply(
    context: &JourneyContext,
    original: &OriginalCustody,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    continue_original: bool,
    close_prepared: bool,
) -> Option<worth_query_host::facade::primary_graph::WorthQueryReleasedApplicationEffect> {
    let original_identity = original.identity.clone();
    let original_attempt = original.attempt.clone();
    let original_meaning = original.native_meaning.clone();
    let predecessor = original.predecessor.clone();
    let (_, live) = scope();
    let mut home = super::original::reopen(context, execution);
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
    assert_eq!(context.contacts.load(Ordering::SeqCst), 2);
    assert!(matches!(
        set(home.runtime(), &live, 0x9921, 10).unwrap(),
        Mutation::IdempotencyIntentDrift
    ));
    assert_eq!(context.contacts.load(Ordering::SeqCst), 2);
    let mut recovery = effects.into_recovery();
    assert_eq!(recovery.record_handle().identity(), &original_identity);
    if close_prepared {
        let observation = home
            .runtime()
            .runtime()
            .world_operation_control_for_test()
            .observe_recovered_continued_for_test(|| {
                std::panic::panic_any(ContinuedBoundaryReached)
            });
        let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| loop {
            match recovery.continue_original_in_request(&execution).unwrap() {
                Continuation::Pending(custody) => drop(custody),
                other => {
                    panic!("the disposition observer must interrupt before occurrence: {other:?}")
                }
            }
        }));
        let failure =
            interrupted.expect_err("actual control interrupts the acknowledged disposition");
        assert!(
            failure.is::<ContinuedBoundaryReached>(),
            "only the real acknowledged-disposition callback may interrupt this fixture"
        );
        drop(observation);
        drop(recovery);
        home.close(&execution).unwrap_or_else(|refused| {
            panic!("prepared continuation must retire through actual close: {refused:?}")
        });
        home = super::original::reopen(context, execution);
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
        assert_eq!(context.contacts.load(Ordering::SeqCst), 2);
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
    let released = if continue_original {
        None
    } else {
        Some(super::released::observe(&home, context, original))
    };
    if !continue_original {
        home.checkpoint(execution)
            .unwrap_or_else(|cause| panic!("released original checkpoint: {cause:?}"));
    }
    drop(recovery);
    assert_eq!(context.contacts.load(Ordering::SeqCst), 2);
    drop(home);
    released
}
