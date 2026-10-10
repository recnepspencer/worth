use std::sync::{mpsc, Mutex};

use worth_foundational::PartitionIdentity;

use super::{
    admission::BatchDeclaration, input::InputMode, native, run::run_batch, BackendKind, BatchStop,
    KernelFailure,
};
use crate::{
    authority::ExecutionResourceLease,
    tests::{authority, batch, request, TEST_LOCK},
};

/// The existing input-dispatch port encloses evaluation AND its settlement-slot
/// hand-off. Releasing after `execute` returns orders more than kernel bodies.
struct LaterHandedOffFirst<'a>(&'a [u64]);

impl InputMode for LaterHandedOffFirst<'_> {
    type Item = u64;

    fn memory_bytes<R, E>(&self, declaration: &BatchDeclaration) -> Option<u64> {
        declaration.execution_memory_bytes::<u64, R, E>(self.0)
    }

    fn dispatch(
        &mut self,
        lease: Option<&ExecutionResourceLease<'_>>,
        parallel: bool,
        seed: Option<u64>,
        execute: &(impl Fn((usize, u64)) + Sync),
    ) -> usize {
        assert!(parallel);
        assert_eq!(seed, None);
        let (handed_off, release) = mpsc::channel();
        let release = Mutex::new(release);
        native::run_order(lease.unwrap(), &[0, 1], &|index| {
            if index == 0 {
                release.lock().unwrap().recv().unwrap();
            }
            execute((index, self.0[index]));
            if index == 1 {
                handed_off.send(()).unwrap();
            }
        })
    }

    fn discard(&mut self) -> Option<usize> {
        None
    }
}

#[test]
fn least_key_failure_wins_after_larger_key_is_handed_to_settlement() {
    let _serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(2, 2_000, 10)).unwrap();
    let admitted = batch(&[1, 2], 16);
    let outcome = run_batch(
        Some(&lease),
        admitted.declaration(),
        LaterHandedOffFirst(admitted.values()),
        BackendKind::Native,
        &|value, context| -> Result<u64, KernelFailure<&'static str>> {
            context.checkpoint(1)?;
            Err(KernelFailure::Domain(if value == 1 {
                "earlier"
            } else {
                "later"
            }))
        },
        true,
        None,
        None,
    );
    drop(_serial);
    assert_eq!(
        outcome.stop,
        Some(BatchStop::Failure {
            identity: PartitionIdentity::new(1),
            cause: KernelFailure::Domain("earlier"),
        }),
        "settlement must report the least failing key regardless of hand-off order"
    );
    assert_eq!(outcome.prefix_boundary, Some(PartitionIdentity::new(1)));
    assert!(outcome.values.is_empty());
}

#[test]
fn success_values_publish_in_key_order_after_larger_key_is_handed_off() {
    let _serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(2, 2_000, 10)).unwrap();
    let admitted = batch(&[1, 2], 16);
    let outcome = run_batch(
        Some(&lease),
        admitted.declaration(),
        LaterHandedOffFirst(admitted.values()),
        BackendKind::Native,
        &|value, context| {
            context.checkpoint(1)?;
            Ok::<_, KernelFailure<()>>(value * 10)
        },
        true,
        None,
        None,
    );
    drop(_serial);
    assert_eq!(outcome.stop, None);
    assert_eq!(
        outcome.values,
        vec![10, 20],
        "settlement must publish success values in key order regardless of hand-off order"
    );
}
