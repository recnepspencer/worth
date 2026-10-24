//! D2: with a medium attached, a publisher that meets the patch-position
//! reservation held by another publisher's durable I/O waits for it, within
//! its request, instead of deferring. Two commits on different branches both
//! land, in arrival order, at contiguous positions; a waiter whose request is
//! cancelled stops before any effect.

use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::medium::{RelationalDurableMedium, RelationalDurablePublication};
use crate::facade::history::BranchId;
use crate::facade::runtime::RelationalRuntimeApi;
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalRuntime;
use crate::tests::support::*;

/// Failure budget for a gate or observation nobody completes; it is not the
/// ordering mechanism.
const STALL_BUDGET: Duration = Duration::from_secs(30);

/// A medium whose first publication after the test arms it holds until the
/// test opens its gate, so that publisher holds the reservation for as long as
/// the test needs.
#[derive(Debug)]
struct SlowFirstPublication {
    contents: Box<dyn RelationalDurableMedium>,
    gate: Arc<PublicationGate>,
}

#[derive(Debug, Default)]
struct PublicationGate {
    state: Mutex<GateState>,
    changed: Condvar,
}

#[derive(Debug, Default)]
struct GateState {
    armed: bool,
    publishing: bool,
    open: bool,
}

impl PublicationGate {
    fn arm(&self) {
        self.state.lock().unwrap().armed = true;
    }

    fn open(&self) {
        self.state.lock().unwrap().open = true;
        self.changed.notify_all();
    }

    fn wait_until(&self, reached: impl Fn(&GateState) -> bool) {
        let deadline = Instant::now() + STALL_BUDGET;
        let mut state = self.state.lock().unwrap();
        while !reached(&state) {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .expect("the publication gate stalled");
            state = self.changed.wait_timeout(state, remaining).unwrap().0;
        }
    }
}

impl RelationalDurableMedium for SlowFirstPublication {
    fn publish(
        &self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> RelationalDurablePublication {
        let first = {
            let mut state = self.gate.state.lock().unwrap();
            let first = state.armed && !state.publishing;
            state.publishing |= state.armed;
            first
        };
        if first {
            self.gate.changed.notify_all();
            self.gate.wait_until(|state| state.open);
        }
        self.contents.publish(artifact)
    }
}

/// A durable runtime with a committed main, a `feature` fork of it and an
/// armed gate, and the reservation contacts its setup made.
fn slow_runtime() -> (
    RelationalRuntime,
    Arc<TestDurableMedium>,
    Arc<PublicationGate>,
    u64,
) {
    let contents = TestDurableMedium::new();
    let gate = Arc::new(PublicationGate::default());
    let runtime = RelationalRuntimeApi::builder()
        .profile(crate::config::data::RelationalRuntimeProfile::CertificationCore)
        .schema_registry(test_schema_registry())
        .build_durable(Box::new(SlowFirstPublication {
            contents: contents.attachment(),
            gate: Arc::clone(&gate),
        }));
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let seed = create_entity_outcome(&execution, &runtime, "main-seed");
    release_test_commit_snapshot(&runtime, &seed);
    create_branch_from_main(&runtime, "feature");
    gate.arm();
    let contacts = runtime.patch_position_reservation_counters().contacts;
    (runtime, contents, gate, contacts)
}

/// Block until a second publisher has reached the reservation the first
/// holds inside its durable publication.
fn await_second_contact(runtime: &RelationalRuntime, setup_contacts: u64) {
    let deadline = Instant::now() + STALL_BUDGET;
    while runtime.patch_position_reservation_counters().contacts < setup_contacts + 2 {
        assert!(
            Instant::now() < deadline,
            "the second publisher never arrived"
        );
        std::thread::yield_now();
    }
}

#[test]
fn a_durable_publisher_waits_for_the_reservation_instead_of_deferring() {
    let (runtime, contents, gate, setup_contacts) = slow_runtime();
    let runtime = &runtime;
    let (first, second) = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            let request = crate::test_execution_policy::serial_request();
            let execution = worth_execution::ExecutionRequest::serial(&request);
            create_entity_outcome(&execution, runtime, "main-first")
        });
        gate.wait_until(|state| state.publishing);
        let second = scope.spawn(|| {
            let request = crate::test_execution_policy::serial_request();
            let execution = worth_execution::ExecutionRequest::serial(&request);
            create_entity_outcome_on_branch(
                &execution,
                runtime,
                "feature-second",
                BranchId("feature".to_owned()),
            )
        });
        await_second_contact(runtime, setup_contacts);
        gate.open();
        (first.join().unwrap(), second.join().unwrap())
    });

    assert_eq!(first.patch_position(), PatchStreamPosition(2));
    assert_eq!(second.patch_position(), PatchStreamPosition(3));
    assert_eq!(
        contents.positions(),
        vec![
            PatchStreamPosition(1),
            PatchStreamPosition(2),
            PatchStreamPosition(3)
        ]
    );
    let counters = runtime.patch_position_reservation_counters();
    assert_eq!(counters.deferrals, 0, "nobody deferred");
    assert_eq!(counters.assignments, 3);
    release_test_commit_snapshot(runtime, &first);
    release_test_commit_snapshot(runtime, &second);
}

#[test]
fn a_cancelled_waiter_stops_before_any_effect() {
    let (runtime, contents, gate, setup_contacts) = slow_runtime();
    let runtime = &runtime;
    let feature_before = runtime
        .history()
        .branch_head(&BranchId("feature".to_owned()));
    let cancellation = worth_execution::CancellationSource::new();
    let (first, cancelled) = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            let request = crate::test_execution_policy::serial_request();
            let execution = worth_execution::ExecutionRequest::serial(&request);
            create_entity_outcome(&execution, runtime, "main-first")
        });
        gate.wait_until(|state| state.publishing);
        let token = cancellation.token();
        let cancelled = scope.spawn(move || {
            let request = worth_execution::SerialRequest::from_memory(
                worth_execution::SerialMemoryBudget::new(None),
                token,
                None,
            );
            let execution = worth_execution::ExecutionRequest::serial(&request);
            let mut transaction = crate::tests::support::test_owner_begin_transaction_for_branch(
                runtime,
                BranchId("feature".to_owned()),
            );
            transaction
                .push_batch(
                    batch_create("feature-cancelled"),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            transaction.commit(&execution, runtime)
        });
        await_second_contact(runtime, setup_contacts);
        cancellation.cancel();
        let cancelled = cancelled.join().unwrap();
        gate.open();
        (first.join().unwrap(), cancelled)
    });

    assert!(
        matches!(
            cancelled,
            Err(crate::transactions::data::TransactionCommitError::Execution { .. })
        ),
        "a cancelled waiter returns its request stop: {cancelled:?}"
    );
    assert_eq!(first.patch_position(), PatchStreamPosition(2));
    assert_eq!(
        contents.positions(),
        vec![PatchStreamPosition(1), PatchStreamPosition(2)]
    );
    assert_eq!(contents.offered().len(), 2, "the waiter offered nothing");
    assert_eq!(
        runtime
            .history()
            .branch_head(&BranchId("feature".to_owned())),
        feature_before,
        "the waiter installed nothing"
    );
    let counters = runtime.patch_position_reservation_counters();
    assert_eq!((counters.deferrals, counters.assignments), (0, 2));
    release_test_commit_snapshot(runtime, &first);

    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let retried = create_entity_outcome_on_branch(
        &execution,
        runtime,
        "feature-retried",
        BranchId("feature".to_owned()),
    );
    assert_eq!(retried.patch_position(), PatchStreamPosition(3));
    release_test_commit_snapshot(runtime, &retried);
}
