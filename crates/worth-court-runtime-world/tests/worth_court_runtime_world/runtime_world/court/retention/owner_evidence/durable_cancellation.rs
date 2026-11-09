//! World cancellation reaches a durable candidate waiting on its actual Signal cell.
use super::super::artifact_source::Fixture;
use super::*;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::{Duration, Instant},
};
#[test]
fn durable_candidate_cancelled_while_waiting_never_runs_its_body() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut fixture = Fixture::journal(&request);
    let root = match fixture.offer(&request, "primary") {
        WorldDurableBootstrapOutcome::Performed(value) => value.product_branch().clone(),
        other => panic!("bootstrap: {other:?}"),
    };
    let cancellation = RuntimeWorldCancellationSource::new();
    let port = fixture.world.publication_port();
    let opened = match port
        .prepare_query_publication_with_signal(
            &request,
            root.clone(),
            fixture.recorder.program_source(&root),
            None,
            &cancellation.token(),
            None,
        )
        .unwrap()
        .advance(&request)
    {
        WorldOpenAttemptOutcome::Opened(value) => value,
        other => panic!("open: {other:?}"),
    };
    let prepared = opened
        .prepare_signal(&fixture.native, &request, None)
        .unwrap();
    let source = fixture.signal.source_port();
    let basis = fixture.signal.basis_port();
    let control = source.owner_operation_control_for_test().unwrap();
    let pause = control.arm_pause_once_for_test(
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::DurableCandidateCellWait,
    );
    let invoked = Arc::new(AtomicBool::new(false));
    let invoked_by_body = Arc::clone(&invoked);
    let (held_tx, held_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let bound = Duration::from_secs(30);
    std::thread::scope(|scope| {
        let worker_port = port.clone();
        let token = cancellation.token();
        let operation = scope.spawn(move || {
            let funding = crate::test_execution_policy::serial_request();
            worker_port.execute_durable_with_signal(
                ExecutionRequest::serial(&funding),
                prepared,
                &mut (),
                &token,
                move |_| {
                    invoked_by_body.store(true, Ordering::SeqCst);
                    Ok(())
                },
            )
        });
        assert!(pause.wait_until_reached(bound));
        let expected = root.basis().signal_basis();
        let held_source = source.clone();
        let holder = scope.spawn(move || {
            held_source
                .hold_branch_cell_for_test(expected, || {
                    held_tx.send(()).unwrap();
                    release_rx.recv_timeout(bound).unwrap();
                })
                .unwrap()
        });
        held_rx.recv_timeout(bound).unwrap();
        let waits = basis
            .owner_service_cost_snapshot()
            .unwrap()
            .target_cell_waits();
        pause.release();
        let deadline = Instant::now() + bound;
        while basis
            .owner_service_cost_snapshot()
            .unwrap()
            .target_cell_waits()
            == waits
        {
            assert!(
                Instant::now() < deadline,
                "candidate reaches the real held-cell wait"
            );
            std::thread::yield_now();
        }
        cancellation.cancel();
        release_tx.send(()).unwrap();
        holder.join().unwrap();
        let outcome = operation.join().unwrap();
        assert!(matches!(
            outcome,
            WorldDurablePublicationOutcome::ProductUnpublished(_)
        ));
        assert!(!invoked.load(Ordering::SeqCst));
    });
    assert!(matches!(
        source
            .inspect_associated_advances(&request, |_| std::ops::ControlFlow::Break(()))
            .unwrap(),
        std::ops::ControlFlow::Continue(())
    ));
    drop((source, basis, control, pause, port, root));
    // A cancelled candidate leaves the committed World open in original custody.
    // Dropping the outcome cannot settle that business record. Reopen and
    // explicitly release its exact OpenOnly original through the durable door.
    let fixture = fixture.reopen(&request);
    let recovery = fixture.world.recovery_port();
    let page = fixture
        .world
        .inspection_port()
        .recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let custody = recovery
        .inspect_recovered_effects(page.rows()[0].handle())
        .unwrap();
    assert_eq!(custody.native_receipt(), None);
    assert!(matches!(
        recovery.release_recovered_effects(&request, custody),
        WorldRecoveredReleaseOutcome::Released(_)
    ));
    drop((page, recovery));
    fixture.finish(&request);
}
