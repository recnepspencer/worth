//! A capped scope refuses a payload past its capacity to the exact byte, and the
//! payload's own measure and destructor stay inside the scope's panic boundary.
use super::*;
use crate::backend::{run_scope, KernelFailure};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

struct Hostile {
    bytes: u64,
    measure_panics: bool,
    drops: Arc<AtomicU64>,
}

impl ChargedBytes for Hostile {
    fn additional_charged_bytes(&self) -> u64 {
        assert!(!self.measure_panics, "hostile measure");
        self.bytes
    }
}

impl Drop for Hostile {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
        if !std::thread::panicking() {
            panic!("hostile destructor");
        }
    }
}

fn hostile(bytes: u64, measure_panics: bool, drops: &Arc<AtomicU64>) -> Hostile {
    Hostile {
        bytes,
        measure_panics,
        drops: Arc::clone(drops),
    }
}

#[test]
fn capacity_refused_result_is_disposed_once_under_the_panic_boundary() {
    let disposed = Arc::new(AtomicU64::new(0));
    let outcome = run_scope::<Hostile, (), _>(None, 0, 0, |_| Ok(hostile(1, false, &disposed)));
    assert!(matches!(
        outcome.result,
        Err(ScopeStop::Failure(KernelFailure::ResultCapacityExceeded))
    ));
    assert_eq!(disposed.load(Ordering::SeqCst), 1);
}

#[test]
fn capacity_refused_domain_error_is_disposed_once_under_the_panic_boundary() {
    let disposed = Arc::new(AtomicU64::new(0));
    let outcome = run_scope::<(), Hostile, _>(None, 0, 0, |_| {
        Err(KernelFailure::Domain(hostile(1, false, &disposed)))
    });
    assert!(matches!(
        outcome.result,
        Err(ScopeStop::Failure(KernelFailure::ResultCapacityExceeded))
    ));
    assert_eq!(disposed.load(Ordering::SeqCst), 1);
}

#[test]
fn capped_scope_reports_a_panicking_measure_as_a_contained_panic() {
    let disposed = Arc::new(AtomicU64::new(0));
    let outcome =
        run_scope::<Hostile, (), _>(None, 0, 1 << 20, |_| Ok(hostile(1, true, &disposed)));
    assert!(matches!(
        outcome.result,
        Err(ScopeStop::Failure(KernelFailure::Panic))
    ));
    assert_eq!(disposed.load(Ordering::SeqCst), 1);
}

#[test]
fn result_and_error_capacity_refuse_only_past_the_exact_cap() {
    let run_result = |cap, length: usize| {
        run_scope::<Vec<u8>, (), _>(None, 0, cap, move |_| Ok(Vec::with_capacity(length))).result
    };
    let run_error = |cap, length: usize| {
        run_scope::<(), Vec<u8>, _>(None, 0, cap, move |_| {
            Err(KernelFailure::Domain(Vec::with_capacity(length)))
        })
        .result
    };
    assert!(run_result(64, 64).is_ok());
    assert!(matches!(
        run_result(63, 64),
        Err(ScopeStop::Failure(KernelFailure::ResultCapacityExceeded))
    ));
    assert!(matches!(
        run_error(64, 64),
        Err(ScopeStop::Failure(KernelFailure::Domain(_)))
    ));
    assert!(matches!(
        run_error(63, 64),
        Err(ScopeStop::Failure(KernelFailure::ResultCapacityExceeded))
    ));
}
