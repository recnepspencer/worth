//! Focused owner tests for D14 discard serialization, cancellation, and settlement failure.
use super::*;
use crate::{ApplicationHomeCheckpointDenial, ApplicationHomeCloseOutcome};
use std::sync::Arc;
use worth_store::physical_runtime::production::PhysicalMutationCheckpoint;
use worth_store::physical_runtime::{
    AnchoredRecordOperationFailure, AnchoredRecordPublicationFailure,
};

fn open_composed_home(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> WorldReadmittedApplicationHome<(), (), (), (), ()> {
    let binding = worth_query_testkit_installation::facade::installation::application_home_binding::admitted_home_binding();
    let build = worth_foundational::facade::ApplicationBuildIdentity::parse(
        "11111111-1111-4111-8111-111111111111",
    )
    .unwrap();
    let opening = crate::ApplicationHomeRequest::new(
        location.to_owned(),
        crate::tests::native_schema::definition(),
        binding,
        build,
    );
    let home = execution
        .in_scope(|_| {
            let recovered = crate::recover_home(opening, execution).unwrap();
            let admitted = crate::admit_home(recovered, execution).unwrap();
            crate::readmit_relational(admitted, execution)
                .unwrap()
                .consume(|home| home)
        })
        .unwrap();
    let (builder, bridge, signal, _attachment) = recipe(&home, execution);
    let world = super::readmit_attached_world(&home, builder, bridge, execution).unwrap();
    let checkpoint = signal.checkpoint_port();
    super::finish_attachment(home, world, None, Some(signal), checkpoint).consume(|home| home)
}

fn serial_with_token(token: worth_execution::CancellationToken) -> worth_execution::SerialRequest {
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(16 << 20)),
        token,
        None,
    )
}

fn assert_live_discarded_and_close(
    home: WorldReadmittedApplicationHome<(), (), (), (), ()>,
    expected_gen: u64,
    execution: &ExecutionRequest<'_, '_>,
) {
    let (selected, raw_gen) =
        crate::checkpoint::read_with_anchor_generation(&home.home.context.serving, execution)
            .unwrap();
    assert!(selected.is_none(), "discard marker must yield no selected manifest");
    assert_eq!(raw_gen, expected_gen);
    assert!(home.home.context.checkpoint.is_derived_discarded());
    assert_eq!(home.home.context.checkpoint.acknowledged_native(), [0, 0]);
    assert!(matches!(
        home.checkpoint(execution),
        Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded)
    ));
    let op = home.checkpoint_operation(Default::default()).unwrap();
    assert!(matches!(
        op.after_advancement(execution),
        Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded)
    ));
    drop(op);
    let closed = home
        .close(execution)
        .unwrap_or_else(|cause| panic!("close after discard: {:?}", cause.reason()));
    match closed {
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded),
            ..
        } => {}
        ApplicationHomeCloseOutcome::Closed { checkpoint, .. } => {
            panic!("expected Closed(DerivedStateDiscarded), got Closed({checkpoint:?})")
        }
        ApplicationHomeCloseOutcome::CloseRefused { reason, .. } => {
            panic!("expected Closed(DerivedStateDiscarded), got CloseRefused({reason:?})")
        }
    }
}

#[test]
fn discard_serializes_with_pending_checkpoint_and_defers_fresh_checkpoint_to_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let policy = serial_with_token(worth_execution::CancellationToken::new());
    let execution = ExecutionRequest::serial(&policy);
    let home = open_composed_home(directory.path(), &execution);
    crate::tests::native_schema::commit(home.relational().runtime(), &execution, "owner-discard");
    let first = home.checkpoint(&execution).unwrap();
    assert!(first.covered().0 > 0);

    let guard = home.home.context.checkpoint.begin_discard().unwrap();
    assert!(matches!(
        home.checkpoint(&execution),
        Err(ApplicationHomeCheckpointDenial::DiscardInProgress)
    ));
    assert!(matches!(
        home.discard_derived_for_test(&execution),
        Err(ApplicationHomeCheckpointDenial::DiscardInProgress)
    ));
    drop(guard);
    assert!(!home.home.context.checkpoint.is_derived_discarded());

    crate::tests::native_schema::commit(home.relational().runtime(), &execution, "pending-cp");
    let cancel = worth_execution::CancellationSource::new();
    let cancel_policy = serial_with_token(cancel.token());
    let cancel_execution = ExecutionRequest::serial(&cancel_policy);
    let op = home.checkpoint_operation(Default::default()).unwrap();
    let refused = op
        .checkpoint_with_native_ack_observer_for_test(
            &cancel_execution,
            Box::new(move || cancel.cancel()),
        )
        .unwrap_err();
    drop(op);
    assert!(matches!(
        refused,
        ApplicationHomeCheckpointDenial::RecoveryRequired(_)
    ));
    assert!(matches!(
        home.discard_derived_for_test(&execution),
        Err(ApplicationHomeCheckpointDenial::RecoveryRequired(_))
    ));
    let recovered_cp = home.recover_checkpoint(&execution).unwrap();
    assert_eq!(recovered_cp.generation(), first.generation() + 1);

    let discard_gen = home.discard_derived_for_test(&execution).unwrap();
    assert_eq!(discard_gen, recovered_cp.generation() + 1);
    assert_live_discarded_and_close(home, discard_gen, &execution);

    let reopened = open_composed_home(directory.path(), &execution);
    assert!(!reopened.home.context.checkpoint.is_derived_discarded());
    assert_eq!(reopened.home.context.checkpoint.acknowledged_native(), [0, 0]);
    let resumed_cp = reopened.checkpoint(&execution).unwrap();
    assert_eq!(resumed_cp.generation(), discard_gen + 1);
    assert!(reopened.home.context.checkpoint.acknowledged_native()[0] > 0);
    let closed = reopened
        .close(&execution)
        .unwrap_or_else(|cause| panic!("close after reopen: {:?}", cause.reason()));
    match closed {
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Ok(cp), ..
        } => assert_eq!(cp.generation(), resumed_cp.generation() + 1),
        _ => panic!("expected normal close checkpoint after reopen"),
    }
}

#[test]
fn discard_accepted_pre_cutover_cancellation_allows_same_generation_retry() {
    let directory = tempfile::tempdir().unwrap();
    let policy = serial_with_token(worth_execution::CancellationToken::new());
    let execution = ExecutionRequest::serial(&policy);
    let home = open_composed_home(directory.path(), &execution);
    crate::tests::native_schema::commit(home.relational().runtime(), &execution, "pre-cutover");
    let first = home.checkpoint(&execution).unwrap();
    let coverage_before = home.home.context.checkpoint.acknowledged_native();
    assert!(coverage_before[0] > 0);

    let gate = home
        .home
        .context
        .serving
        .pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeEffectCutover);
    let cancel = worth_execution::CancellationSource::new();
    let cancel_token = cancel.token();
    let serving = Arc::clone(&home.home.context.serving);
    let owner = Arc::clone(&home.home.context.checkpoint);
    let (stopped_tx, stopped_rx) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let cancel_policy = serial_with_token(cancel_token);
        let cancel_exec = ExecutionRequest::serial(&cancel_policy);
        let notify = move || {
            let _ = stopped_tx.send(());
        };
        match crate::checkpoint::discard_for_test::discard_with_test_hooks_for_test(
            &serving,
            &owner,
            &cancel_exec,
            false,
            Some(&notify),
        ) {
            Err(ApplicationHomeCheckpointDenial::Publication(
                AnchoredRecordPublicationFailure::OwnedPublication(custody),
            )) => {
                assert!(custody.observe().terminal().is_none());
                assert!(custody.observe().step_terminal().is_some_and(|t| {
                    t.settlement_failure().is_none()
                        && matches!(
                            t.outcome(),
                            worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(
                                _
                            )
                        )
                }));
            }
            other => panic!("expected pre-cutover ProvenNoEffect failure, got {other:?}"),
        }
    });
    assert!(gate.await_arrival(), "discard must reach BeforeEffectCutover");
    cancel.cancel();
    stopped_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("owner must enter stopped.settle() before releasing BeforeEffectCutover");
    gate.release();
    worker.join().unwrap();

    let (selected, raw_gen) =
        crate::checkpoint::read_with_anchor_generation(&home.home.context.serving, &execution)
            .unwrap();
    assert!(selected.is_some(), "pre-cutover cancel must keep prior checkpoint");
    drop(selected);
    assert_eq!(raw_gen, first.generation());
    assert!(!home.home.context.checkpoint.is_derived_discarded());
    assert_eq!(
        home.home.context.checkpoint.acknowledged_native(),
        coverage_before
    );

    // Same-generation retry with a fresh monotonic nonce must succeed cleanly.
    let retried_gen = home.discard_derived_for_test(&execution).unwrap();
    assert_eq!(retried_gen, first.generation() + 1);
    assert_live_discarded_and_close(home, retried_gen, &execution);
}

#[test]
fn discard_selected_before_owner_settlement_cancellation_marks_owner_discarded() {
    let directory = tempfile::tempdir().unwrap();
    let policy = serial_with_token(worth_execution::CancellationToken::new());
    let execution = ExecutionRequest::serial(&policy);
    let home = open_composed_home(directory.path(), &execution);
    crate::tests::native_schema::commit(home.relational().runtime(), &execution, "post-cutover");
    let first = home.checkpoint(&execution).unwrap();

    let gate = home
        .home
        .context
        .serving
        .pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeTerminalFinalization);
    let cancel = worth_execution::CancellationSource::new();
    let cancel_token = cancel.token();
    let serving = Arc::clone(&home.home.context.serving);
    let owner = Arc::clone(&home.home.context.checkpoint);
    let (stopped_tx, stopped_rx) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let cancel_policy = serial_with_token(cancel_token);
        let cancel_exec = ExecutionRequest::serial(&cancel_policy);
        let notify = move || {
            let _ = stopped_tx.send(());
        };
        let outcome = crate::checkpoint::discard_for_test::discard_with_test_hooks_for_test(
            &serving,
            &owner,
            &cancel_exec,
            false,
            Some(&notify),
        );
        assert!(
            matches!(outcome, Err(ApplicationHomeCheckpointDenial::Execution(_))),
            "expected cancelled execution stop after selection, got {outcome:?}"
        );
    });
    assert!(
        gate.await_arrival(),
        "discard must reach BeforeTerminalFinalization"
    );
    cancel.cancel();
    stopped_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("owner must enter stopped.settle() before releasing BeforeTerminalFinalization");
    gate.release();
    worker.join().unwrap();
    assert_live_discarded_and_close(home, first.generation() + 1, &execution);
}

#[test]
fn discard_selected_with_settlement_failure_reconciles_step_and_stopped_paths() {
    for stopped_path in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let policy = serial_with_token(worth_execution::CancellationToken::new());
        let execution = ExecutionRequest::serial(&policy);
        let home = open_composed_home(directory.path(), &execution);
        crate::tests::native_schema::commit(
            home.relational().runtime(),
            &execution,
            "settlement-fail",
        );
        let first = home.checkpoint(&execution).unwrap();

        let assert_settlement_failure =
            move |outcome: Result<u64, ApplicationHomeCheckpointDenial>| {
            match outcome {
                Err(ApplicationHomeCheckpointDenial::Publication(
                    AnchoredRecordPublicationFailure::OwnedPublication(custody),
                )) => {
                    assert!(
                        matches!(custody.cause(), AnchoredRecordOperationFailure::StepSettlement(_)),
                        "expected typed StepSettlement failure, got {:?}",
                        custody.cause()
                    );
                    assert!(
                        custody.observe().terminal().is_some(),
                        "selected receipt must be retained on settlement failure"
                    );
                }
                other => panic!(
                    "expected OwnedPublication(StepSettlement) on stopped_path={stopped_path}, got {other:?}"
                ),
            }
        };

        if stopped_path {
            let gate = home
                .home
                .context
                .serving
                .pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeTerminalFinalization);
            let cancel = worth_execution::CancellationSource::new();
            let cancel_token = cancel.token();
            let serving = Arc::clone(&home.home.context.serving);
            let owner = Arc::clone(&home.home.context.checkpoint);
            let (stopped_tx, stopped_rx) = std::sync::mpsc::sync_channel(1);
            let worker = std::thread::spawn(move || {
                let cancel_policy = serial_with_token(cancel_token);
                let cancel_exec = ExecutionRequest::serial(&cancel_policy);
                let notify = move || {
                    let _ = stopped_tx.send(());
                };
                let outcome =
                    crate::checkpoint::discard_for_test::discard_with_test_hooks_for_test(
                        &serving,
                        &owner,
                        &cancel_exec,
                        true,
                        Some(&notify),
                    );
                assert_settlement_failure(outcome);
            });
            assert!(gate.await_arrival());
            cancel.cancel();
            stopped_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .expect("owner must enter stopped.settle() before releasing settlement failure");
            gate.release();
            worker.join().unwrap();
        } else {
            let outcome = crate::checkpoint::discard_for_test::discard_with_test_hooks_for_test(
                &home.home.context.serving,
                &home.home.context.checkpoint,
                &execution,
                true,
                None,
            );
            assert_settlement_failure(outcome);
        }
        assert_live_discarded_and_close(home, first.generation() + 1, &execution);
    }
}
