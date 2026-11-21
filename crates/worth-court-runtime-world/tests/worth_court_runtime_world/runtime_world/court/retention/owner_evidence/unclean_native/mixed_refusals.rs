//! A mixed original's planned Signal leg refuses typed before any Signal
//! effect when it no longer names the installed definition, is never begun
//! twice in one process, and a closure Signal leg never starts on a durable
//! medium.
use super::mixed::Cut;
use super::*;
use worth_relational::facade::mvcc::RelationalTransactionIntent;

/// A Native-only mixed original, recovered by a fresh process.
fn native_only(request: &ExecutionRequest<'_, '_>) -> (Fixture, Model) {
    let mut fixture = Fixture::bridge_journal(request);
    match fixture.offer(request, "primary") {
        WorldDurableBootstrapOutcome::Performed(_) => {}
        other => panic!("real bootstrap: {other:?}"),
    }
    let mut model = Model::bootstrapped(&fixture);
    assert!(model.attempt(&fixture, request, Step::Mixed(Cut::NativeOnly)));
    let fixture = model.reopen(fixture, request);
    (fixture, model)
}

fn recovered(fixture: &Fixture) -> WorldRecoveredUnpublishedEffects {
    let page = fixture
        .world
        .inspection_port()
        .recovery_page(None, maximum())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    fixture
        .world
        .recovery_port()
        .inspect_recovered_effects(page.rows()[0].handle())
        .unwrap()
}

#[test]
fn a_planned_leg_altered_to_name_another_definition_refuses_before_any_signal_effect() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let (fixture, model) = native_only(&request);
    fixture
        .recorder
        .rewrite_latest(WorldDurableJournal::Custody, |mut bytes| {
            assert_eq!(bytes[52], 10, "the latest custody row is the planned leg");
            rename_definition(&mut bytes);
            Some(bytes)
        });
    let fixture = fixture.reopen(&request);
    let recovery = fixture.world.recovery_port();
    let outcome = recovery.continue_recovered_effects(&request, recovered(&fixture));
    let WorldRecoveredContinuationOutcome::Refused {
        cause: WorldRecoveredContinuationDenial::SignalLeg(WorldSignalLegDenial::Bridge(cause)),
        custody,
    } = outcome
    else {
        panic!("another definition's leg must refuse: {outcome:?}")
    };
    assert!(matches!(
        *cause,
        worth_runtime_bridge::facade::BridgeDurableDefinitionPublicationDenial::IdentityOrCorrespondenceChanged
    ));
    assert_eq!(
        fixture.signal().published_advance_count().unwrap(),
        model.signal_base + model.signal_commits,
        "no Signal effect precedes the refusal"
    );
    assert_eq!(
        fixture.native.history().immutable_commit_count(),
        model.native_commits
    );
    assert!(
        matches!(
            recovery.release_recovered_effects(&request, custody),
            WorldRecoveredReleaseOutcome::Released(_)
        ),
        "release stays available while no Signal leg has begun"
    );
    fixture.finish(&request);
}

#[test]
fn an_indeterminate_recovered_signal_leg_waits_for_reopen_and_is_never_begun_twice() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let (fixture, mut model) = native_only(&request);
    fixture.hold_signal_advances();
    let recovery = fixture.world.recovery_port();
    let outcome = recovery.continue_recovered_effects(&request, recovered(&fixture));
    let WorldRecoveredContinuationOutcome::Refused {
        cause:
            WorldRecoveredContinuationDenial::Admission(RuntimeWorldRecoveryDenial::ReopenRequired),
        custody,
    } = outcome
    else {
        panic!("an unknown Signal fate waits for reopen: {outcome:?}")
    };
    assert_eq!(
        custody.next_actions(),
        &[
            ProductUnpublishedNextAction::Inspect,
            ProductUnpublishedNextAction::ReopenRequired
        ]
    );
    let outcome = recovery.continue_recovered_effects(&request, custody);
    let WorldRecoveredContinuationOutcome::Refused { custody, .. } = outcome else {
        panic!("the lost leg is never begun again here: {outcome:?}")
    };
    assert!(matches!(
        recovery.release_recovered_effects(&request, custody),
        WorldRecoveredReleaseOutcome::AdmissionRefused {
            cause: RuntimeWorldRecoveryDenial::ReopenRequired,
            ..
        }
    ));
    drop(recovery);
    // Nothing became durable: the next process still holds a Native-only original.
    let fixture = model.reopen(fixture, &request);
    model.decide_all(&fixture, &request, false);
    assert!(model.originals.is_empty());
    fixture.finish(&request);
}

#[test]
fn a_closure_signal_leg_on_a_durable_mixed_original_refuses_before_any_owner_effect() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut fixture = Fixture::journal(&request);
    match fixture.offer(&request, "primary") {
        WorldDurableBootstrapOutcome::Performed(_) => {}
        other => panic!("real bootstrap: {other:?}"),
    }
    let head = root(&fixture);
    let native_before = fixture.native.history().immutable_commit_count();
    let signal_before = fixture.signal().published_advance_count().unwrap();
    let port = fixture.world.publication_port();
    let cancellation = RuntimeWorldCancellationSource::new();
    let WorldOpenAttemptOutcome::Opened(opened) = port
        .prepare_query_publication_with_signal(
            &request,
            head.clone(),
            fixture.recorder.program_source(&head),
            Some(RelationalTransactionIntent::ordinary()),
            &cancellation.token(),
            None,
        )
        .unwrap()
        .advance(&request)
    else {
        panic!("actual mixed open")
    };
    let mut transaction =
        worth_testkit_relational::aspect_storage::publications::begin_main_transaction(
            &fixture.native,
        );
    transaction
        .push_batch(
            worth_testkit_relational::durable_publication::batch_create("closure"),
            request.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction.validate(&request, &fixture.native).unwrap();
    let prepared = opened
        .prepare_signal(&fixture.native, &request, Some(proposal))
        .unwrap();
    let mut body = false;
    let outcome =
        port.execute_durable_with_signal(request, prepared, &mut (), &cancellation.token(), |_| {
            body = true;
            Ok(())
        });
    assert!(matches!(
        outcome,
        WorldDurablePublicationOutcome::RecoveryRequired {
            cause: WorldDurablePublicationDenial::UnrecordableSignalLeg,
            ..
        }
    ));
    drop((outcome, head, port));
    assert!(!body, "the closure never runs");
    assert_eq!(
        fixture.native.history().immutable_commit_count(),
        native_before
    );
    assert_eq!(
        fixture.signal().published_advance_count().unwrap(),
        signal_before
    );
    // The committed open is kept and releases as proven no-effect.
    let fixture = fixture.reopen(&request);
    let effects = recovered(&fixture);
    assert_eq!(effects.owner_effect_count(), 0);
    assert!(matches!(
        fixture
            .world
            .recovery_port()
            .release_recovered_effects(&request, effects),
        WorldRecoveredReleaseOutcome::Released(_)
    ));
    fixture.finish(&request);
}

/// Rename the planned leg's Bridge contract to another definition, keeping
/// Bridge's and World's frames valid so only Bridge admission can refuse it.
fn rename_definition(row: &mut [u8]) {
    let name = super::super::super::artifact_source::bridge_owner::DEFINITION.as_bytes();
    let at = row
        .windows(name.len())
        .position(|part| part == name)
        .expect("the leg names its definition once");
    row[at + name.len() - 1] ^= 1;
    let leg = row
        .windows(8)
        .position(|part| part == b"WBRGPLEG")
        .expect("Bridge's planned-leg frame");
    let mut hash = sha2::Sha256::new();
    hash.update(&row[leg + 8..leg + 20]);
    hash.update(&row[leg + 52..]);
    let hash: [u8; 32] = hash.finalize().into();
    row[leg + 20..leg + 52].copy_from_slice(&hash);
    super::super::journey::rehash(row);
}
