//! A payload's own code, measure and destructor alike, runs inside the scope's
//! panic boundary on every settlement path, and each payload is disposed once.
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use worth_execution::{
    CancellationSource, ChargedBytes, ExecutionMemoryReservation, ExecutionRequest,
    ExecutionWorkCeiling, LeaseDenial, MapKernelContext, MapKernelFailure, MapKernelStop,
    MemoryLimitDenial, ScopeOutcome, ScopeStop, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
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

fn panicking_destructor(bytes: u64, drops: &Arc<AtomicU64>) -> Hostile {
    Hostile {
        bytes,
        measure_panics: false,
        drops: Arc::clone(drops),
    }
}

fn panicking_measure(drops: &Arc<AtomicU64>) -> Hostile {
    Hostile {
        bytes: 1,
        measure_panics: true,
        drops: Arc::clone(drops),
    }
}

fn drops() -> Arc<AtomicU64> {
    Arc::new(AtomicU64::new(0))
}

struct Scope {
    budget: SerialMemoryBudget,
    request: SerialRequest,
    memory: ExecutionMemoryReservation,
}

fn scope(limit: Option<u64>, cancellation: &CancellationSource) -> Scope {
    let budget = SerialMemoryBudget::new(limit);
    let request = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let memory = budget.reserve(0).unwrap();
    Scope {
        budget,
        request,
        memory,
    }
}

impl Scope {
    fn run_measured<R: ChargedBytes, E: ChargedBytes>(
        &mut self,
        kernel: impl FnOnce(&mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>>,
    ) -> ScopeOutcome<R, E> {
        ExecutionRequest::serial(&self.request).run_measured_result(
            ExecutionWorkCeiling::new(None),
            0,
            &mut self.memory,
            |context, _| kernel(context),
        )
    }
}

#[test]
fn measured_scope_settles_its_reservation_after_a_panicking_result_measure() {
    let mut scope = scope(None, &CancellationSource::new());
    let disposed = drops();
    let stopped =
        scope.run_measured(|_| Ok::<_, MapKernelFailure<()>>(panicking_measure(&disposed)));
    assert!(matches!(
        stopped.result,
        Err(ScopeStop::Failure(MapKernelFailure::Panic))
    ));
    assert_eq!(disposed.load(Ordering::SeqCst), 1);
    assert_eq!(scope.memory.bytes(), 0);
    drop(scope.memory);
    assert_eq!(scope.budget.charged_memory_bytes(), 0);
}

#[test]
fn measured_scope_settles_its_reservation_after_a_panicking_error_measure() {
    let mut scope = scope(None, &CancellationSource::new());
    let disposed = drops();
    let stopped = scope
        .run_measured(|_| Err::<(), _>(MapKernelFailure::Domain(panicking_measure(&disposed))));
    assert!(matches!(
        stopped.result,
        Err(ScopeStop::Failure(MapKernelFailure::Panic))
    ));
    assert_eq!(disposed.load(Ordering::SeqCst), 1);
    assert_eq!(scope.memory.bytes(), 0);
}

#[test]
fn measured_admission_refusal_survives_a_panicking_destructor() {
    let mut scope = scope(Some(128 * 1024), &CancellationSource::new());
    let disposed = drops();
    let stopped = scope
        .run_measured(|_| Ok::<_, MapKernelFailure<()>>(panicking_destructor(1 << 40, &disposed)));
    assert!(matches!(
        stopped.result,
        Err(ScopeStop::Admission(LeaseDenial::MemoryExhausted(
            MemoryLimitDenial::Capacity { .. }
        )))
    ));
    assert_eq!(disposed.load(Ordering::SeqCst), 1);
    assert_eq!(scope.memory.bytes(), 0);
}

#[test]
fn control_stop_disposes_a_completed_payload_once_under_the_panic_boundary() {
    let cancellation = CancellationSource::new();
    let mut scope = scope(None, &cancellation);
    let disposed = drops();
    let stopped = scope.run_measured(|context| {
        cancellation.cancel();
        assert_eq!(context.checkpoint(0), Err(MapKernelStop::Cancelled));
        Ok::<_, MapKernelFailure<()>>(panicking_destructor(1, &disposed))
    });
    assert!(matches!(
        stopped.result,
        Err(ScopeStop::Failure(MapKernelFailure::Stop(
            MapKernelStop::Cancelled
        )))
    ));
    assert_eq!(disposed.load(Ordering::SeqCst), 1);
    assert_eq!(scope.memory.bytes(), 0);
}

#[test]
fn stopped_entry_disposes_its_unrun_kernel_once_under_the_panic_boundary() {
    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let scope = scope(None, &cancellation);
    let disposed = drops();
    let captured = panicking_destructor(0, &disposed);
    let denied = ExecutionWorkCeiling::new(None).run_serial(&scope.request, move || {
        let _owned = captured;
    });
    assert!(matches!(
        denied,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
    ));
    assert_eq!(disposed.load(Ordering::SeqCst), 1);
    assert_eq!(scope.budget.charged_memory_bytes(), 0);
}

#[test]
fn denied_entry_disposes_its_unrun_kernel_once_under_the_panic_boundary() {
    let scope = scope(Some(0), &CancellationSource::new());
    let disposed = drops();
    let captured = panicking_destructor(0, &disposed);
    let denied = ExecutionWorkCeiling::new(None).run_serial(&scope.request, move || {
        let _owned = captured;
    });
    assert!(matches!(
        denied,
        Err(WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(
            MemoryLimitDenial::Capacity { .. }
        )))
    ));
    assert_eq!(disposed.load(Ordering::SeqCst), 1);
}
