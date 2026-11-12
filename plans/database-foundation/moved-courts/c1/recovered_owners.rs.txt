//! Both actual owners are terminal before cancellation; only World movement remains.
use super::super::artifact_source::Fixture;
use super::*;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
#[test]
fn recovered_signal_only_original_keeps_its_owner_and_continues_without_reexecution() {
    run(false);
}
#[test]
fn recovered_mixed_original_keeps_both_owners_and_continues_without_reexecution() {
    run(true);
}
fn run(mixed: bool) {
    let funding = crate::test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&funding);
    let mut fixture = Fixture::journal(request);
    let root = match fixture.offer(request, "primary") {
        WorldDurableBootstrapOutcome::Performed(value) => value.product_branch().clone(),
        other => panic!("actual bootstrap: {other:?}"),
    };
    let port = fixture.world.publication_port();
    let cancellation = RuntimeWorldCancellationSource::new();
    let opened = match port
        .prepare_query_publication_with_signal(
            request,
            root.clone(),
            fixture.recorder.program_source(&root),
            mixed.then(RelationalTransactionIntent::ordinary),
            &cancellation.token(),
            None,
        )
        .unwrap()
        .advance(request)
    {
        WorldOpenAttemptOutcome::Opened(opened) => opened,
        other => panic!("actual original open: {other:?}"),
    };
    let commit = opened.reserved_commit_identity();
    let proposal = if mixed {
        let mut transaction =
            worth_testkit_relational::aspect_storage::publications::begin_main_transaction(
                &fixture.native,
            );
        transaction
            .push_batch(
                worth_testkit_relational::durable_publication::batch_create("mixed-original"),
                request.allocation_policy(),
            )
            .unwrap();
        Some(transaction.validate(request, &fixture.native).unwrap())
    } else {
        None
    };
    let prepared = opened
        .prepare_signal(&fixture.native, request, proposal)
        .unwrap();
    let control = fixture.world.world_operation_control_for_test();
    let pause = control.pause_before_product_compare_for_test(std::num::NonZeroUsize::MIN);
    let body_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let calls = body_calls.clone();
    let token = cancellation.token();
    let outcome = std::thread::scope(|scope| {
        let operation = scope.spawn(move || {
            let funding = crate::test_execution_policy::serial_request();
            port.execute_durable_with_signal(
                ExecutionRequest::serial(&funding),
                prepared,
                &mut (),
                &token,
                move |_| {
                    calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(())
                },
            )
        });
        assert!(pause.wait_until_reached(std::time::Duration::from_secs(30)));
        cancellation.cancel();
        pause.release();
        operation.join().unwrap()
    });
    let WorldDurablePublicationOutcome::RecoveryRequired {
        cause: WorldDurablePublicationDenial::CancelledAfterEffect,
        custody,
    } = outcome
    else {
        panic!("both original owners must precede cancellation: {outcome:?}")
    };
    assert_eq!(custody.owner_effect_count(), 1 + usize::from(mixed));
    let attempt = custody.attempt_identity().clone();
    let original_native = fixture.native.history().historical_latest_commit();
    let original_signal = fixture.signal.published_advance_count().unwrap();
    drop((custody, root, control, pause));
    let fixture = fixture.reopen(request);
    let recovery = fixture.world.recovery_port();
    let page = fixture
        .world
        .inspection_port()
        .recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let effects = recovery
        .inspect_recovered_effects(page.rows()[0].handle())
        .unwrap();
    assert_eq!(effects.attempt_identity().ordinal(), attempt.ordinal());
    assert_eq!(
        effects.native_receipt().is_some(),
        mixed,
        "Signal-only must fabricate no Native receipt"
    );
    assert_eq!(effects.owner_effect_count(), 1 + usize::from(mixed));
    assert_eq!(
        effects.live_obligation_count(),
        1 + 2 + 1 + effects.owner_effect_count(),
        "seat, expected component/history observation and the actual original owner loans"
    );
    assert!(effects
        .next_actions()
        .contains(&ProductUnpublishedNextAction::ContinueOriginalPublication));
    fixture.recorder.hold_publication(true);
    let WorldRecoveredContinuationOutcome::Pending(effects) =
        recovery.continue_recovered_effects(request, effects)
    else {
        panic!("the actual World journal retains the prepared original continuation")
    };
    assert_eq!(effects.owner_effect_count(), 1 + usize::from(mixed));
    assert_eq!(effects.live_obligation_count(),1+2+1+effects.owner_effect_count()+2+2+2+1,
        "the prepared continuation owns history and origin pin pairs, the head transfer and history reservation");
    fixture.recorder.hold_publication(false);
    let outcome = recovery.continue_recovered_effects(request, effects);
    let WorldRecoveredContinuationOutcome::Published { commit: actual } = outcome else {
        panic!("the exact original must continue: {outcome:?}")
    };
    assert_eq!(actual.ordinal(), commit.ordinal());
    assert_eq!(
        fixture.native.history().historical_latest_commit(),
        original_native
    );
    assert_eq!(
        fixture.signal.published_advance_count().unwrap(),
        original_signal
    );
    assert_eq!(body_calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert!(fixture
        .world
        .inspection_port()
        .recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .is_empty());
    drop((recovery, page));
    fixture.finish(request);
}
