use std::sync::{Arc, TryLockError};

use crate::branch::reference_test_fixture::{self, RealReferenceFixture};
use crate::branch::{ProductBranchObservation, RuntimeWorldBootstrapOutcome};
use crate::lifecycle::RuntimeWorldCloseDenial;
use crate::publication::RuntimeWorldCancellationSource;
use crate::publication::{CompositePublicationIntent, NoEffectCause};

type TestOwner = super::super::RuntimeWorldOwnerRoot<(), (), (), (), ()>;

fn setup() -> (
    RealReferenceFixture,
    Arc<TestOwner>,
    ProductBranchObservation,
) {
    let mut fixture = reference_test_fixture::real_fixture(8, 8);
    let owner = Arc::new(
        TestOwner::new(fixture.owner_inputs(
            super::bootstrap_budgets(),
            crate::lifecycle::RuntimeWorldClock::from_source(super::FixedClock),
        ))
        .expect("managed owner construction"),
    );
    let performed = match owner.bootstrap_root(fixture.bootstrap_intent()) {
        RuntimeWorldBootstrapOutcome::Performed(performed) => performed,
        RuntimeWorldBootstrapOutcome::NoEffect(no_effect) => {
            panic!("bootstrap unexpectedly denied: {:?}", no_effect.cause())
        }
    };
    (fixture, owner, performed.product_branch().clone())
}

/// One Signal-only publication reservation off the given head. Preparation
/// and reservation are a single owner step, so a racing caller cannot hold a
/// lowered plan without the capacity that authorizes it.
fn prepare(
    owner: &TestOwner,
    expected: ProductBranchObservation,
) -> Result<
    crate::publication::PreparedCompositePublicationWithSignal,
    crate::publication::NoEffectCompositePublication,
> {
    let cancellation = RuntimeWorldCancellationSource::new();
    crate::lifecycle::RuntimeWorldPreparationService::prepare_publication(
        owner,
        expected,
        CompositePublicationIntent::with_signal(None),
        &cancellation.token(),
        None,
    )
}

/// Wait for a reservation worker to reach the bootstrap admission lock.
/// Reservation admission holds that lock across the operation ledger it then
/// blocks on, so the hold is a real observable rather than a sampled instant.
/// The budget is wall-clock, not a yield count: several Runtime World
/// worktrees build and test on one machine, so a scheduler starved of cores
/// must not be reported as a lost race. The wait fails by name, never hangs.
#[track_caller]
fn wait_until_bootstrap_is_held(owner: &TestOwner) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while std::time::Instant::now() < deadline {
        match owner.state.bootstrap.try_lock() {
            Err(TryLockError::WouldBlock) => return,
            Err(TryLockError::Poisoned(error)) => {
                panic!("bootstrap lock poisoned: {error}")
            }
            Ok(guard) => drop(guard),
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("reservation worker never reached the bootstrap admission lock");
}

/// Wait until a close worker is queued on the operation ledger inside close
/// admission. Close publishes its queued waiter immediately before it takes
/// that lock, so entering admission is a production observable rather than an
/// elapsed interval: this waits for the state itself, never for a settle
/// window, and fails by name inside a bounded budget.
#[track_caller]
pub(super) fn wait_until_close_is_admitting(owner: &TestOwner) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while owner.close_admission_waiters() == 0 {
        if std::time::Instant::now() >= deadline {
            panic!("close worker never queued on the owner close admission ledger");
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

#[test]
fn publication_attempts_own_independent_phase_state() {
    let (_fixture, owner, expected) = setup();
    let first = prepare(&owner, expected.clone()).expect("first attempt reserves");
    let second =
        prepare(&owner, expected).expect("second attempt is not serialized behind the first");

    assert_eq!(owner.state.operation.active(), 2);
    drop(first);
    assert_eq!(owner.state.operation.active(), 1);
    drop(second);
    assert_eq!(owner.state.operation.active(), 0);
}

#[test]
fn reserve_and_close_have_one_atomic_winner_in_both_lock_orders() {
    let (_fixture, owner, expected) = setup();
    let ledger_gate = owner
        .state
        .operation
        .state
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let reserve_owner = Arc::clone(&owner);
    let reserve_head = expected.clone();
    let reserve = std::thread::spawn(move || prepare(reserve_owner.as_ref(), reserve_head));
    wait_until_bootstrap_is_held(&owner);
    let close_owner = Arc::clone(&owner);
    let close = std::thread::spawn(move || close_owner.close());
    drop(ledger_gate);

    let attempt = reserve
        .join()
        .expect("reserve worker does not panic")
        .expect("reserve wins after holding bootstrap admission");
    assert_eq!(
        close
            .join()
            .expect("close worker does not panic")
            .expect_err("the reserving winner blocks the close drain"),
        RuntimeWorldCloseDenial::AlreadyClosing
    );
    drop(attempt);
    let _report = owner
        .close()
        .expect("close succeeds after the winner settles");

    let (_fixture, owner, expected) = setup();
    let ledger_gate = owner
        .state
        .operation
        .state
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let close_owner = Arc::clone(&owner);
    let close = std::thread::spawn(move || close_owner.close());
    wait_until_close_is_admitting(&owner);
    // A queued mutex waiter is not an acquired lock: Mutex promises no FIFO
    // fairness. Hold the later close-contract lock so close can acquire and
    // keep the operation ledger before the competing reservation starts.
    let contract_gate = owner
        .state
        .close
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    drop(ledger_gate);
    wait_until_close_holds_operation_admission(&owner);
    let reserve_owner = Arc::clone(&owner);
    let reserve = std::thread::spawn(move || prepare(reserve_owner.as_ref(), expected));
    drop(contract_gate);

    let _report = close
        .join()
        .expect("close worker does not panic")
        .expect("close wins after holding operation admission");
    let denial = reserve
        .join()
        .expect("reserve worker does not panic")
        .expect_err("closed owner denies the trailing reservation");
    assert_eq!(denial.cause(), NoEffectCause::OwnerUnavailable);
    assert_eq!(owner.state.operation.active(), 0);
}

/// The queued-close count falls to zero only after close owns the ledger.
/// The caller holds the later close-contract lock, keeping this observable
/// stable until the competing operation has been started.
#[track_caller]
pub(super) fn wait_until_close_holds_operation_admission(owner: &TestOwner) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while std::time::Instant::now() < deadline {
        let queued = owner.close_admission_waiters();
        let held = matches!(
            owner.state.operation.state.try_lock(),
            Err(TryLockError::WouldBlock)
        );
        if queued == 0 && held {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("close never held operation admission across its drain within 10s");
}
