use std::sync::mpsc::{self, sync_channel, TryRecvError};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use super::world::empty_runtime;
use worth_relational::facade::branch::RelationalOwnerLifecycleObservation;
use worth_relational::facade::mvcc::{
    RelationalOperationControl, RelationalPublicationOutcome, RelationalTransactionIntent,
};
use worth_relational::facade::transactions::WorkerIntentBatch;

const LIFECYCLE_BOUND: Duration = Duration::from_secs(2);
const OWNER_CLOSE_START_FAIL_SAFE: Duration = Duration::from_secs(6);

#[test]
fn descriptive_owner_lifecycle_observes_open_closing_and_closed_without_authority() {
    let runtime = empty_runtime();
    let services = runtime.owner_component_services();
    let lifecycle_port = services.lifecycle_port();
    assert_eq!(
        lifecycle_port.owner_lifecycle_observation(),
        RelationalOwnerLifecycleObservation::Open
    );

    let (reached, release) = (Arc::new(Barrier::new(2)), Arc::new(Barrier::new(2)));
    let control = RelationalOperationControl::uninterrupted()
        .with_critical_section_pause(Arc::clone(&reached), Arc::clone(&release));
    let basis = services
        .basis_port()
        .admit_branch_basis(&runtime.main_branch_identity())
        .expect("owner service admits the exact main basis");
    let mut transaction = runtime
        .begin_branch_transaction_with_control(
            &basis,
            RelationalTransactionIntent::ordinary(),
            control,
        )
        .expect("controlled publication transaction opens");
    transaction
        .push_batch(WorkerIntentBatch::new("observe-owner-closing"))
        .expect("lifecycle observation batch stays in budget");
    let candidate = services
        .preparation_port()
        .prepare_branch_transaction(transaction)
        .expect("controlled candidate prepares canonically");

    let (publication_tx, publication_rx) = sync_channel(1);
    let publication_port = services.publication_port();
    std::thread::spawn(move || {
        let _ = publication_tx.send(publication_port.compare_and_publish(candidate));
    });

    let (owner_close_start_tx, owner_close_start_rx) = sync_channel(1);
    let (owner_dropped_tx, owner_dropped_rx) = sync_channel(1);
    std::thread::spawn(move || {
        let _ = owner_close_start_rx.recv_timeout(OWNER_CLOSE_START_FAIL_SAFE);
        drop(runtime);
        let _ = owner_dropped_tx.send(());
    });
    assert_pause_reached(Arc::clone(&reached), Arc::clone(&release));
    let owner_close_started = owner_close_start_tx.send(()).is_ok();
    let closing_reached = wait_until(LIFECYCLE_BOUND, || {
        lifecycle_port.owner_lifecycle_observation() == RelationalOwnerLifecycleObservation::Closing
    });
    let owner_waited_for_operation = owner_dropped_rx.try_recv() == Err(TryRecvError::Empty);

    let pause_released = release_controlled_pause(release);
    let publication = publication_rx.recv_timeout(LIFECYCLE_BOUND);
    let publication_performed =
        matches!(&publication, Ok(RelationalPublicationOutcome::Performed(_)));
    drop(publication);
    let owner_dropped = owner_dropped_rx.recv_timeout(LIFECYCLE_BOUND);

    assert!(
        owner_close_started,
        "owner close controller disappeared before release"
    );
    assert!(
        closing_reached,
        "owner never exposed Closing within the bound"
    );
    assert!(
        owner_waited_for_operation,
        "owner drop completed while admitted publication remained parked"
    );
    assert_eq!(
        pause_released,
        Ok(()),
        "publication pause release must finish within the bound"
    );
    assert!(
        publication_performed,
        "parked admitted publication must complete after release"
    );
    assert_eq!(owner_dropped, Ok(()), "owner close must drain in bound");
    assert_eq!(
        lifecycle_port.owner_lifecycle_observation(),
        RelationalOwnerLifecycleObservation::Closed
    );
}

fn release_controlled_pause(release: Arc<Barrier>) -> Result<(), mpsc::RecvTimeoutError> {
    let (released_tx, released_rx) = sync_channel(1);
    std::thread::spawn(move || {
        release.wait();
        let _ = released_tx.send(());
    });
    released_rx.recv_timeout(LIFECYCLE_BOUND)
}

fn assert_pause_reached(reached: Arc<Barrier>, release: Arc<Barrier>) {
    let (arrived_tx, arrived_rx) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        reached.wait();
        let _ = arrived_tx.send(());
    });
    if arrived_rx.recv_timeout(LIFECYCLE_BOUND).is_err() {
        std::thread::spawn(move || {
            release.wait();
        });
        panic!("publication never reached its controlled critical section");
    }
}

fn wait_until(timeout: Duration, mut observed: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if observed() {
            return true;
        }
        std::thread::yield_now();
    }
    observed()
}
