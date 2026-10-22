use super::*;
use crate::tests::{authority, request, TEST_LOCK};
use crate::{CancellationSource, MapKernelStop, MapStop, NestedComputationStop};
use std::{
    cell::{Cell, RefCell},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

// Deliberately Send-only, with no ChargedBytes implementation. The payload is
// larger than the authority's entire memory ceiling, while framework slots fit.
struct Payload {
    value: Cell<Vec<u8>>,
    dropped: Arc<AtomicUsize>,
}
impl Drop for Payload {
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::SeqCst);
    }
}
#[derive(Debug)]
struct Failure(RefCell<String>);

fn map(dropped: &Arc<AtomicUsize>) -> ExecutionMap<Payload, (), NativePayload> {
    ExecutionMap::from_native_partitions(
        (1..=3)
            .map(|id| {
                (
                    PartitionIdentity::new(id),
                    Payload {
                        value: Cell::new(vec![id as u8; 8192]),
                        dropped: Arc::clone(dropped),
                    },
                )
            })
            .collect(),
    )
    .unwrap()
}

#[test]
fn send_only_payloads_keep_canonical_order_without_heap_invoices() {
    let _guard = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(3, 2_000, 100)).unwrap();
    for seed in [None, Some(7)] {
        let dropped = Arc::new(AtomicUsize::new(0));
        let kernel = |value: Payload, context: &mut MapKernelContext<'_, '_>| {
            context.poll()?;
            Ok::<_, MapKernelFailure<Failure>>(value)
        };
        let outcome = match seed {
            None => map(&dropped).run_owned_native(Some(&lease), kernel),
            Some(seed) => {
                let (outcome, order) =
                    map(&dropped).run_owned_native_perturbed_for_test(Some(&lease), seed, kernel);
                assert_eq!(order.len(), 3);
                outcome
            }
        };
        match outcome {
            MapOutcome::Complete { values, report } => {
                assert_eq!(
                    values.iter().map(|v| v.value.take()[0]).collect::<Vec<_>>(),
                    [1, 2, 3]
                );
                assert_eq!(report.charged_work(), 0);
                assert_eq!(dropped.load(Ordering::SeqCst), 0);
                drop(values);
            }
            MapOutcome::Stopped { reason, .. } => panic!("native payload failed: {reason:?}"),
        }
        assert_eq!(dropped.load(Ordering::SeqCst), 3);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}

#[test]
fn native_domain_failure_keeps_the_canonical_prefix_and_input_custody() {
    let _guard = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(3, 2_000, 100)).unwrap();
    let dropped = Arc::new(AtomicUsize::new(0));
    let outcome = map(&dropped).run_owned_native(Some(&lease), |value, context| {
        context.poll()?;
        let id = value.value.take()[0];
        if id > 1 {
            return Err(MapKernelFailure::Domain(Failure(RefCell::new(format!(
                "failure-{id}"
            )))));
        }
        Ok(value)
    });
    match outcome {
        MapOutcome::Stopped {
            completed_prefix,
            boundary,
            reason:
                MapStop::Failure {
                    identity,
                    cause: MapKernelFailure::Domain(error),
                },
            ..
        } => {
            assert_eq!(boundary, Some(PartitionIdentity::new(2)));
            assert_eq!(identity, PartitionIdentity::new(2));
            assert_eq!(error.0.into_inner(), "failure-2");
            assert_eq!(completed_prefix.len(), 1);
            assert_eq!(dropped.load(Ordering::SeqCst), 2);
            drop(completed_prefix);
        }
        _ => panic!("native domain refusal lost canonical prefix"),
    }
    assert_eq!(dropped.load(Ordering::SeqCst), 3);
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn native_poll_retains_cancelled_request_even_when_kernel_returns_success() {
    let _guard = TEST_LOCK.lock().unwrap();
    let cancel = CancellationSource::new();
    let mut requested = request(1, 2_000, 100);
    requested.cancellation = cancel.token();
    let lease = authority().request_lease(requested).unwrap();
    let map = ExecutionMap::from_native_partitions(BTreeMap::from([(
        PartitionIdentity::new(1),
        Cell::new(1),
    )]))
    .unwrap();
    let outcome = map.run_owned_native(Some(&lease), |value, context| {
        assert_eq!(context.poll(), Ok(()));
        cancel.cancel();
        assert_eq!(context.poll(), Err(MapKernelStop::Cancelled));
        assert_eq!(context.poll(), Err(MapKernelStop::Cancelled));
        Ok::<_, MapKernelFailure<Failure>>(value)
    });
    assert!(
        matches!(outcome, MapOutcome::Stopped { completed_prefix, reason: MapStop::Failure { cause: MapKernelFailure::Stop(MapKernelStop::Cancelled), .. }, report, .. } if completed_prefix.is_empty() && report.charged_work() == 0)
    );
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn native_poll_preserves_recoverable_computational_stop_ownership() {
    let _guard = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let parent =
        ExecutionMap::from_native_partitions(BTreeMap::from([(PartitionIdentity::new(1), ())]))
            .unwrap();
    let outcome = parent.run_owned_native(Some(&lease), |(), context| {
        let child =
            ExecutionMap::from_native_partitions(BTreeMap::from([(PartitionIdentity::new(1), ())]))
                .unwrap();
        let child = child.run_owned_native(Some(&lease), |(), _| {
            Err::<(), _>(MapKernelFailure::Domain(Failure(RefCell::new(
                "child".into(),
            ))))
        });
        assert!(matches!(
            child,
            MapOutcome::Stopped {
                reason: MapStop::Failure {
                    cause: MapKernelFailure::Domain(_),
                    ..
                },
                ..
            }
        ));
        assert_eq!(context.poll(), Err(MapKernelStop::NestedStopped));
        assert_eq!(
            context.take_nested_computation_stop(),
            Some(NestedComputationStop::DomainFailure)
        );
        assert_eq!(context.poll(), Ok(()));
        Ok::<_, MapKernelFailure<Failure>>(())
    });
    assert!(
        matches!(outcome, MapOutcome::Complete { values, report } if values.len() == 1 && report.charged_work() == 0)
    );
    assert_eq!(lease.charged_memory_bytes(), 0);
}
