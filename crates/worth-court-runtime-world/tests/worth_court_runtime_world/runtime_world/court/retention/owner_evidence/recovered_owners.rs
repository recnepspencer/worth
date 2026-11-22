//! Both actual owners are terminal before cancellation; only World movement remains.
use super::super::artifact_source::Fixture;
use super::*;
#[test]
fn recovered_signal_only_original_keeps_its_owner_and_continues_without_reexecution() {
    run(false, false);
}
/// The mixed original runs World's definition path, the only one a durable
/// medium admits for a Native and a Signal effect.
#[test]
fn recovered_mixed_original_keeps_both_owners_and_continues_without_reexecution() {
    run(true, false);
}
/// D18 still refuses a Signal effect whose recorded open is gone: nothing
/// joins it to an original, so it never reaches service.
#[test]
fn a_signal_effect_without_its_recorded_open_refuses_before_serving() {
    run(false, true);
}
fn run(mixed: bool, orphan: bool) {
    let funding = crate::test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&funding);
    let mut fixture = if mixed {
        Fixture::bridge_journal(request)
    } else {
        Fixture::journal(request)
    };
    let root = match fixture.offer(request, "primary") {
        WorldDurableBootstrapOutcome::Performed(value) => value.product_branch().clone(),
        other => panic!("actual bootstrap: {other:?}"),
    };
    let port = fixture.world.publication_port();
    let cancellation = RuntimeWorldCancellationSource::new();
    let (original, commit) = if mixed {
        let original = super::mixed_definition::prepare(&fixture, request, &root, "mixed-original");
        let commit = original.identities[1];
        (Original::Definition(original), commit)
    } else {
        let opened = match port
            .prepare_query_publication_with_signal(
                request,
                root.clone(),
                fixture.recorder.program_source(&root),
                None,
                &cancellation.token(),
                None,
            )
            .unwrap()
            .advance(request)
        {
            WorldOpenAttemptOutcome::Opened(opened) => opened,
            other => panic!("actual original open: {other:?}"),
        };
        let commit = opened.reserved_commit_identity().ordinal();
        let prepared = opened
            .prepare_signal(&fixture.native, request, None)
            .unwrap();
        (Original::Closure(prepared), commit)
    };
    let control = fixture.world.world_operation_control_for_test();
    let pause = control.pause_before_product_compare_for_test(std::num::NonZeroUsize::MIN);
    let body_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let calls = body_calls.clone();
    let token = cancellation.token();
    let outcome = std::thread::scope(|scope| {
        let operation = scope.spawn(move || {
            let funding = crate::test_execution_policy::serial_request();
            let request = ExecutionRequest::serial(&funding);
            match original {
                Original::Definition(original) => {
                    super::mixed_definition::publish(&port, &request, original, &token)
                }
                Original::Closure(prepared) => port.execute_durable_with_signal(
                    request,
                    prepared,
                    &mut (),
                    &token,
                    move |_| {
                        calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        Ok(())
                    },
                ),
            }
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
    let original_signal = fixture.signal().published_advance_count().unwrap();
    drop((custody, root, control, pause));
    if orphan {
        fixture
            .recorder
            .rewrite_latest(WorldDurableJournal::Custody, |bytes| {
                assert_eq!(bytes[52], 3, "the latest custody row is the original open");
                None
            });
        let denial = match fixture.try_reopen(request) {
            Ok(_) => panic!("an orphan Signal effect cannot reach service"),
            Err(denial) => denial,
        };
        assert!(
            matches!(
                denial,
                WorldReadmissionDenial::UncleanRecoveryUnsupported(UncleanRecoveryUnsupported {
                    owner: DurableOwnerFamily::Signal,
                    ..
                })
            ),
            "{denial:?}"
        );
        return;
    }
    let fixture = fixture.reopen(request);
    let generation = mixed.then(|| super::mixed_definition::served_generation(&fixture));
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
    assert_eq!(actual.ordinal(), commit);
    assert_eq!(
        fixture.native.history().historical_latest_commit(),
        original_native
    );
    assert_eq!(
        fixture.signal().published_advance_count().unwrap(),
        original_signal
    );
    // The closure leg's body runs once; the definition leg has no body, so its
    // non-re-execution evidence is the unchanged Native, Signal and generation
    // counts around continuation.
    if !mixed {
        assert_eq!(body_calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
    if let Some(generation) = generation {
        assert_eq!(
            super::mixed_definition::served_generation(&fixture),
            generation,
            "continuation publishes the readmitted definition, never a second one"
        );
    }
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

enum Original {
    Definition(super::mixed_definition::MixedOriginal),
    Closure(WorldPreparedPublicationWithSignal),
}
