use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::atomic::{AtomicUsize, Ordering},
};

use super::{
    authority, map, request, ChargedBytes, MapKernelFailure, MapOutcome, MapStop, TEST_LOCK,
};
use crate::{backend::run_scope, ExecutionResourceLease, LeaseDenial};

fn charged(lease: &ExecutionResourceLease<'_>) -> u64 {
    let budget = lease.policy().budget().charged_memory_bytes();
    match lease.reserve_memory(budget) {
        Ok(held) => {
            drop(held);
            0
        }
        Err(denial) => budget - denial.admitted,
    }
}

#[test]
fn taking_inline_entry_releases_lineage_in_order() {
    inline_entries([false, true]);
}

#[test]
fn taking_inline_refusal_releases_lineage_in_order() {
    inline_entries([true, false]);
}

fn inline_entries(refusals: [bool; 2]) {
    let _lock = TEST_LOCK.lock().unwrap();
    let busy = authority().request_lease(request(4, 2_000, 100)).unwrap();
    let workers = busy.try_reserve(4, 0).unwrap();
    for owned in [false, true] {
        for refusal in refusals {
            let lease = authority()
                .request_lease(request(1, if refusal { 8 } else { 2_000 }, 100))
                .unwrap();
            let hold = lease.reserve_memory(8).unwrap();
            assert_eq!(charged(&lease), 8);
            let expected = super::memory_model::bytes::<u64, u64, ()>(
                1,
                0,
                (std::mem::size_of::<Vec<u64>>()
                    + std::mem::size_of::<(worth_foundational::PartitionIdentity, Vec<u64>)>()
                    + std::mem::size_of::<u64>()) as u64,
                1,
                1,
                true,
            );
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                if owned {
                    map([1_u64]).run_owned_taking(Some(&lease), hold, |value, _| {
                        assert_eq!(charged(&lease), expected);
                        Ok::<_, MapKernelFailure<()>>(value)
                    })
                } else {
                    map([1_u64]).run_taking(Some(&lease), hold, |value, _| {
                        assert_eq!(charged(&lease), expected);
                        Ok::<_, MapKernelFailure<()>>(*value)
                    })
                }
            }));
            assert!(
                outcome.is_ok(),
                "inline release panicked: owned={owned}, refusal={refusal}"
            );
            let outcome = outcome.unwrap();
            assert_eq!(charged(&lease), 0, "inline entry released all ledger bytes");
            assert_eq!(outcome.report().charged_work(), 0);
            if refusal {
                assert!(matches!(
                    outcome,
                    MapOutcome::Stopped {
                        reason: MapStop::Admission(LeaseDenial::MemoryExhausted(_)),
                        ..
                    }
                ));
            } else {
                assert!(matches!(outcome, MapOutcome::Complete { .. }));
            }
        }
    }
    drop(workers);
}

struct Accounting<'a> {
    calls: &'a AtomicUsize,
    panic: bool,
}
impl ChargedBytes for Accounting<'_> {
    fn additional_charged_bytes(&self) -> u64 {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert!(!self.panic, "accounting must follow rejection");
        0
    }
}

#[test]
fn rejected_modes_do_not_account_inputs() {
    let _lock = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    for owned in [false, true] {
        let calls = AtomicUsize::new(0);
        let scope = run_scope(Some(&lease), 0, 0, |_| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                let batch = map([Accounting {
                    calls: &calls,
                    panic: true,
                }]);
                if owned {
                    batch.run_owned(None, |_, _| Ok::<_, MapKernelFailure<()>>(()))
                } else {
                    batch.run(None, |_, _| Ok::<_, MapKernelFailure<()>>(()))
                }
            }));
            assert!(matches!(
                result,
                Ok(MapOutcome::Stopped {
                    reason: MapStop::Admission(LeaseDenial::UnrelatedNestedLease),
                    ..
                })
            ));
            Ok::<_, MapKernelFailure<()>>(())
        });
        assert!(scope.result.is_err()); // The nested denial is carried by the parent meter.
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}

#[test]
fn rejected_prepared_parent_does_not_account_inputs() {
    let _lock = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let calls = AtomicUsize::new(0);
    let mut prepared = None;
    let scope = run_scope(Some(&lease), 0, 0, |_| {
        let child = lease.child(request(1, 2_000, 100)).unwrap();
        prepared = Some(
            map([Accounting {
                calls: &calls,
                panic: false,
            }])
            .prepare_run::<(), ()>(child)
            .unwrap(),
        );
        Ok::<_, MapKernelFailure<()>>(())
    });
    assert!(scope.result.is_ok());
    calls.store(0, Ordering::SeqCst);
    let result = prepared
        .unwrap()
        .run(|_, _| Ok::<_, MapKernelFailure<()>>(()));
    assert!(matches!(
        result,
        MapOutcome::Stopped {
            reason: MapStop::Admission(LeaseDenial::UnrelatedNestedLease),
            ..
        }
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

struct AccountingPanic<'a> {
    lease: &'a ExecutionResourceLease<'a>,
    drops: &'a AtomicUsize,
    violations: &'a AtomicUsize,
}
impl ChargedBytes for AccountingPanic<'_> {
    fn additional_charged_bytes(&self) -> u64 {
        panic!("accounting panic")
    }
}
impl Drop for AccountingPanic<'_> {
    fn drop(&mut self) {
        if charged(self.lease) != (2 * std::mem::size_of::<Self>()) as u64 {
            self.violations.fetch_add(1, Ordering::SeqCst);
        }
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn accounting_unwind_destroys_inputs_under_caller_hold() {
    let _lock = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let drops = AtomicUsize::new(0);
    let violations = AtomicUsize::new(0);
    let hold = lease
        .reserve_memory((2 * std::mem::size_of::<AccountingPanic<'_>>()) as u64)
        .unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| {
        map((0..2).map(|_| AccountingPanic {
            lease: &lease,
            drops: &drops,
            violations: &violations,
        }))
        .run_owned_taking(Some(&lease), hold, |_, _| Ok::<_, MapKernelFailure<()>>(()))
    }));
    assert!(result.is_err());
    assert_eq!(drops.load(Ordering::SeqCst), 2);
    assert_eq!(
        violations.load(Ordering::SeqCst),
        0,
        "hold released before accounting unwind destroyed inputs"
    );
    assert_eq!(charged(&lease), 0);
}

struct DropPanic<'a> {
    index: usize,
    drops: &'a [AtomicUsize],
}
impl ChargedBytes for DropPanic<'_> {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl Drop for DropPanic<'_> {
    fn drop(&mut self) {
        self.drops[self.index].fetch_add(1, Ordering::SeqCst);
        assert!(self.index > 1, "input destructor panic");
    }
}

#[test]
fn destructor_panics_settle_and_remaining_inputs_are_destroyed() {
    let _lock = TEST_LOCK.lock().unwrap();
    for stop in 0..3 {
        let source = crate::CancellationSource::new();
        let mut req = request(1, if stop == 2 { 128 } else { 2_000 }, 100);
        req.cancellation = source.token();
        let lease = authority().request_lease(req).unwrap();
        if stop == 1 {
            source.cancel();
        }
        let hold = lease
            .reserve_memory((3 * std::mem::size_of::<DropPanic<'_>>()) as u64)
            .unwrap();
        let drops: Vec<_> = (0..3).map(|_| AtomicUsize::new(0)).collect();
        let result = catch_unwind(AssertUnwindSafe(|| {
            map((0..3).map(|index| DropPanic {
                index,
                drops: &drops,
            }))
            .run_owned_taking(Some(&lease), hold, |_, _| Ok::<_, MapKernelFailure<()>>(()))
        }));
        assert!(
            matches!(
                result,
                Ok(MapOutcome::Stopped {
                    reason: MapStop::Failure {
                        cause: MapKernelFailure::Panic,
                        ..
                    },
                    ..
                })
            ),
            "destructor escaped or lost at stop {stop}"
        );
        match result.unwrap() {
            MapOutcome::Stopped {
                reason: MapStop::Failure { identity, .. },
                boundary,
                ..
            } => {
                assert_eq!(identity, worth_foundational::PartitionIdentity::new(1));
                assert_eq!(
                    boundary,
                    (stop != 2).then(|| worth_foundational::PartitionIdentity::new(1))
                );
            }
            _ => unreachable!(),
        }
        assert!(drops.iter().all(|v| v.load(Ordering::SeqCst) == 1));
        assert_eq!(charged(&lease), 0);
    }
}

#[test]
fn inline_refusal_closes_entry_before_input_ledger_cleanup() {
    struct LedgerCleanup<'a> {
        parent: &'a ExecutionResourceLease<'a>,
        child: &'a ExecutionResourceLease<'a>,
        drops: &'a AtomicUsize,
    }
    impl ChargedBytes for LedgerCleanup<'_> {
        fn additional_charged_bytes(&self) -> u64 {
            0
        }
    }
    impl Drop for LedgerCleanup<'_> {
        fn drop(&mut self) {
            assert_eq!(charged(self.parent), std::mem::size_of::<Self>() as u64);
            assert_eq!(charged(self.child), 0);
            // A legitimate zero-byte hold can remove an otherwise empty child
            // ledger node. The refused entry must no longer depend on it.
            drop(self.child.reserve_memory(0).unwrap());
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }
    let _lock = TEST_LOCK.lock().unwrap();
    let busy = authority().request_lease(request(4, 2_000, 100)).unwrap();
    let workers = busy.try_reserve(4, 0).unwrap();
    let parent = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let child = parent.child(request(1, 1, 100)).unwrap();
    let drops = AtomicUsize::new(0);
    let hold = parent
        .reserve_memory(std::mem::size_of::<LedgerCleanup<'_>>() as u64)
        .unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| {
        map([LedgerCleanup {
            parent: &parent,
            child: &child,
            drops: &drops,
        }])
        .run_owned_taking(Some(&child), hold, |_, _| {
            Ok::<_, MapKernelFailure<()>>(0_u64)
        })
    }));
    assert!(
        result.is_ok(),
        "refused inline entry outlived child ledger cleanup"
    );
    assert!(matches!(
        result.unwrap(),
        MapOutcome::Stopped {
            reason: MapStop::Admission(LeaseDenial::MemoryExhausted(_)),
            ..
        }
    ));
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(charged(&parent), 0);
    assert_eq!(charged(&child), 0);
    drop(workers);
}
