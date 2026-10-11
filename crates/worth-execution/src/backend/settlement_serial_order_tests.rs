//! Settlement discards injected hand-off order even without worker threads.
use std::sync::Mutex;

use worth_foundational::{ExecutionPosture, PartitionIdentity};

use super::{
    admission::BatchDeclaration, input::InputMode, port::BatchOutcome, run::run_batch, BackendKind,
    BatchStop, KernelContext, KernelFailure,
};
use crate::{
    authority::ExecutionResourceLease,
    report::ChargedBytes,
    tests::{authority, batch, request, TEST_LOCK},
};

// Use the same input-dispatch port as the worker hand-off tests. Each execute
// returns after recording its settlement slot; no thread or wait is needed.
struct LaterHandedOffFirst<'a> {
    values: &'a [u64],
    handed_off: &'a Mutex<Vec<u64>>,
}

impl InputMode for LaterHandedOffFirst<'_> {
    type Item = u64;

    fn memory_bytes<R, E>(&self, declaration: &BatchDeclaration) -> Option<u64> {
        declaration.execution_memory_bytes::<u64, R, E>(self.values)
    }

    fn dispatch(
        &mut self,
        _: Option<&ExecutionResourceLease<'_>>,
        parallel: bool,
        seed: Option<u64>,
        execute: &(impl Fn((usize, u64)) + Sync),
    ) -> usize {
        assert!(!parallel, "the hand-off twin must use serial dispatch");
        assert_eq!(seed, None);
        for index in [1, 0] {
            execute((index, self.values[index]));
            self.handed_off.lock().unwrap().push(self.values[index]);
        }
        1
    }

    fn discard(&mut self) -> Option<usize> {
        None
    }
}

fn serial_handoff<E: ChargedBytes + Send>(
    kernel: impl Fn(u64, &mut KernelContext<'_, '_>) -> Result<u64, KernelFailure<E>> + Sync,
) -> BatchOutcome<u64, E> {
    let serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(2, 2_000, 10)).unwrap();
    if cfg!(target_arch = "wasm32") {
        assert_eq!(
            lease.resolved_posture(),
            ExecutionPosture::Serial,
            "wasm authority must resolve the automatic request serial"
        );
    }
    let admitted = batch(&[1, 2], 16);
    let handed_off = Mutex::new(Vec::new());
    let outcome = run_batch(
        Some(&lease),
        admitted.declaration(),
        LaterHandedOffFirst {
            values: admitted.values(),
            handed_off: &handed_off,
        },
        BackendKind::Serial,
        &kernel,
        true,
        None,
        None,
    );
    drop(serial);
    assert_eq!(
        handed_off.into_inner().unwrap(),
        [2, 1],
        "the dispatch fixture must complete the larger key first"
    );
    assert_eq!(outcome.report.resolved_posture(), ExecutionPosture::Serial);
    outcome
}

#[test]
fn serial_handoff_reports_the_least_failing_partition() {
    let outcome = serial_handoff(|value, context| {
        context.checkpoint(1)?;
        Err(KernelFailure::Domain(if value == 1 {
            "earlier"
        } else {
            "later"
        }))
    });
    assert_eq!(
        outcome.stop,
        Some(BatchStop::Failure {
            identity: PartitionIdentity::new(1),
            cause: KernelFailure::Domain("earlier"),
        }),
        "settlement must report the least failing key regardless of serial hand-off order"
    );
    assert_eq!(outcome.prefix_boundary, Some(PartitionIdentity::new(1)));
    assert!(outcome.values.is_empty());
}

#[test]
fn serial_handoff_publishes_success_values_in_canonical_order() {
    let outcome = serial_handoff(|value, context| {
        context.checkpoint(1)?;
        Ok::<_, KernelFailure<()>>(value * 10)
    });
    assert_eq!(outcome.stop, None);
    assert_eq!(
        outcome.values,
        vec![10, 20],
        "settlement must publish success values in key order regardless of serial hand-off order"
    );
}
