//! Focused owner tests for D14 discard serialization, cancellation, and same-open re-checkpoint.
use super::*;
use crate::{ApplicationHomeCheckpointDenial, ApplicationHomeCloseOutcome};

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

#[test]
fn discard_serializes_with_pending_checkpoint_and_defers_fresh_checkpoint_to_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let policy = serial_with_token(worth_execution::CancellationToken::new());
    let execution = ExecutionRequest::serial(&policy);
    let home = open_composed_home(directory.path(), &execution);
    crate::tests::native_schema::commit(home.relational().runtime(), &execution, "owner-discard");
    let first = home.checkpoint(&execution).unwrap();
    assert!(first.covered().0 > 0);

    // 1. Holding the discard slot excludes both checkpoint() and a second discard.
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

    // 2. Pending checkpoint attempt refuses discard_derived_for_test with RecoveryRequired.
    crate::tests::native_schema::commit(home.relational().runtime(), &execution, "pending-cp");
    let cancel = worth_execution::CancellationSource::new();
    let cancel_token = cancel.token();
    let cancel_policy = serial_with_token(cancel_token);
    let cancel_execution = ExecutionRequest::serial(&cancel_policy);
    let op = home.checkpoint_operation(Default::default()).unwrap();
    let refused = op
        .checkpoint_with_native_ack_observer_for_test(
            &cancel_execution,
            Box::new(move || cancel.cancel()),
        )
        .unwrap_err();
    drop(op);
    assert!(
        matches!(refused, ApplicationHomeCheckpointDenial::RecoveryRequired(_)),
        "expected pending checkpoint attempt, got {refused:?}"
    );
    assert!(matches!(
        home.discard_derived_for_test(&execution),
        Err(ApplicationHomeCheckpointDenial::RecoveryRequired(_))
    ));
    assert!(!home.home.context.checkpoint.is_derived_discarded());
    let recovered_cp = home.recover_checkpoint(&execution).unwrap();
    assert_eq!(recovered_cp.generation(), first.generation() + 1);

    // 3. Discard clears live coverage, refuses same-open checkpoint/maintenance
    // with DerivedStateDiscarded (never MissingCarriedMember or overwriting the
    // discard marker), and graceful close preserves the discard so reopen can
    // resume fresh checkpointing cleanly at discard_gen + 1.
    let discard_gen = home.discard_derived_for_test(&execution).unwrap();
    assert_eq!(discard_gen, recovered_cp.generation() + 1);
    assert!(home.home.context.checkpoint.is_derived_discarded());
    assert_eq!(home.home.context.checkpoint.acknowledged_native(), [0, 0]);

    assert!(matches!(
        home.checkpoint(&execution),
        Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded)
    ));
    let op_after = home.checkpoint_operation(Default::default()).unwrap();
    assert!(matches!(
        op_after.after_advancement(&execution),
        Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded)
    ));
    drop(op_after);
    assert!(home.home.context.checkpoint.is_derived_discarded());

    let closed = home
        .close(&execution)
        .unwrap_or_else(|cause| panic!("close after discard: {:?}", cause.reason()));
    assert!(matches!(
        closed,
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded),
            ..
        }
    ));

    let reopened = open_composed_home(directory.path(), &execution);
    assert!(!reopened.home.context.checkpoint.is_derived_discarded());
    assert_eq!(reopened.home.context.checkpoint.acknowledged_native(), [0, 0]);
    let resumed_cp = reopened.checkpoint(&execution).unwrap();
    assert_eq!(resumed_cp.generation(), discard_gen + 1);
    assert!(reopened.home.context.checkpoint.acknowledged_native()[0] > 0);
    let closed_reopened = reopened
        .close(&execution)
        .unwrap_or_else(|cause| panic!("close after reopen checkpoint: {:?}", cause.reason()));
    match closed_reopened {
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Ok(cp), ..
        } => assert_eq!(cp.generation(), resumed_cp.generation() + 1),
        _ => panic!("expected normal close checkpoint after reopen"),
    }
}

#[test]
fn discard_cancellation_reconciles_before_and_after_anchor_selection() {
    let directory = tempfile::tempdir().unwrap();
    let policy = serial_with_token(worth_execution::CancellationToken::new());
    let execution = ExecutionRequest::serial(&policy);
    let home = open_composed_home(directory.path(), &execution);
    crate::tests::native_schema::commit(home.relational().runtime(), &execution, "cancel-discard");
    let first = home.checkpoint(&execution).unwrap();
    let coverage_before = home.home.context.checkpoint.acknowledged_native();
    assert!(coverage_before[0] > 0);

    // Pre-selection cancellation leaves derived_discarded false, coverage intact, and slot Idle.
    let pre_cancel = worth_execution::CancellationSource::new();
    pre_cancel.cancel();
    let pre_policy = serial_with_token(pre_cancel.token());
    let pre_exec = ExecutionRequest::serial(&pre_policy);
    assert!(matches!(
        home.discard_derived_for_test(&pre_exec),
        Err(ApplicationHomeCheckpointDenial::Execution(_))
    ));
    assert!(!home.home.context.checkpoint.is_derived_discarded());
    assert_eq!(
        home.home.context.checkpoint.acknowledged_native(),
        coverage_before
    );

    // Cancellation inside the step observer (racing with publication): settle reconciles
    // the exact physical outcome so live owner state matches the durable anchor.
    let post_cancel = worth_execution::CancellationSource::new();
    let post_policy = serial_with_token(post_cancel.token());
    let post_exec = ExecutionRequest::serial(&post_policy);
    let outcome = crate::checkpoint::discard_for_test::discard_with_step_observer_for_test(
        &home.home.context.serving,
        &home.home.context.checkpoint,
        &post_exec,
        &move || post_cancel.cancel(),
    );
    let (_, raw_gen) = crate::checkpoint::read_with_anchor_generation(
        &home.home.context.serving,
        &execution,
    )
    .unwrap();
    match outcome {
        Ok(discarded_gen) => {
            assert_eq!(discarded_gen, first.generation() + 1);
            assert_eq!(raw_gen, discarded_gen);
            assert!(home.home.context.checkpoint.is_derived_discarded());
            assert_eq!(home.home.context.checkpoint.acknowledged_native(), [0, 0]);
        }
        Err(ApplicationHomeCheckpointDenial::Execution(_)) => {
            if raw_gen == first.generation() + 1 {
                assert!(
                    home.home.context.checkpoint.is_derived_discarded(),
                    "settled selection on cancellation must mark derived_discarded"
                );
                assert_eq!(home.home.context.checkpoint.acknowledged_native(), [0, 0]);
            } else {
                assert_eq!(raw_gen, first.generation());
                assert!(
                    !home.home.context.checkpoint.is_derived_discarded(),
                    "unselected cancellation must leave derived_discarded false"
                );
                assert_eq!(
                    home.home.context.checkpoint.acknowledged_native(),
                    coverage_before
                );
                // Slot was released cleanly; a fresh discard now succeeds.
                let fresh = home.discard_derived_for_test(&execution).unwrap();
                assert_eq!(fresh, first.generation() + 1);
            }
        }
        other => panic!("unexpected cancellation discard outcome: {other:?}"),
    }

    let closed = home
        .close(&execution)
        .unwrap_or_else(|cause| panic!("close after reconciled discard: {:?}", cause.reason()));
    assert!(matches!(
        closed,
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded),
            ..
        }
    ));
}
