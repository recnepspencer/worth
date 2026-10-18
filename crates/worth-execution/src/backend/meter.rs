use std::{
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use crate::{
    authority::{CancellationToken, ExecutionResourceLease, SerialMemoryBudget},
    report::ChargedBytes,
};

thread_local! {
    static ACTIVE_METER: RefCell<Vec<Rc<RefCell<KernelMeter>>>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn active_serial_memory() -> Option<SerialMemoryBudget> {
    ACTIVE_METER.with(|active| {
        active
            .borrow()
            .last()
            .and_then(|meter| meter.borrow().limits.serial_memory.clone())
    })
}

pub(crate) fn active_stop() -> Option<KernelStop> {
    ACTIVE_METER.with(|active| {
        let active = active.borrow();
        let mut meter = active.last()?.borrow_mut();
        let KernelMeter {
            limits,
            nested_stop,
            ..
        } = &mut *meter;
        limits.safe_point(nested_stop).err().or_else(|| {
            (meter.checkpoint_stop == Some(KernelStop::WorkCeiling))
                .then_some(KernelStop::WorkCeiling)
        })
    })
}

mod activity;
mod detached;
pub(crate) use detached::detach_meter_for_test;
mod checkpoint;
mod kernel_receipt;
pub use kernel_receipt::{ExecutionKernelReceipt, ExecutionKernelReceiptStorage};
mod framework_bytes;
mod invoking_scope;
pub(crate) use invoking_scope::run_invoking_scope;
mod nested;
mod nested_stop;
mod prepared_context;
mod run_context;
mod structural_span;
pub(crate) use activity::{enter_certification_activity, enter_retained_memory, has_active_kernel};
pub(crate) use nested::{record_nested, record_nested_outcome};
pub use nested_stop::NestedComputationStop;
use nested_stop::NestedStop;
pub(crate) use nested_stop::NestedStopOwner;
pub(crate) use nested_stop::{request_refusal, request_stop};
pub(crate) use prepared_context::PreparedMeterContext;
pub(crate) use structural_span::StructuralSpanFailure;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelStop {
    Cancelled,
    DeadlineElapsed,
    WorkCounterOverflow,
    WorkCeiling,
    NestedStopped,
}

impl KernelStop {
    pub(crate) fn work_counter_overflow() -> Self {
        request_stop(Self::WorkCounterOverflow)
    }
}

impl ChargedBytes for KernelStop {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum KernelFailure<E> {
    Stop(KernelStop),
    Domain(E),
    Panic,
    ResultCapacityExceeded,
}

impl<E> From<KernelStop> for KernelFailure<E> {
    fn from(value: KernelStop) -> Self {
        Self::Stop(value)
    }
}

#[derive(Clone)]
pub(crate) struct RunLimits {
    tokens: Vec<CancellationToken>,
    deadline: Option<Instant>,
    ceiling: u64,
    ceiling_owner: NestedStopOwner,
    worker_activity: Arc<WorkerActivity>,
    physical_activity: Arc<PhysicalActivity>,
    physical_cell_charged: bool,
    force_serial: bool,
    bound_to_lease: bool,
    /// The memory limit a lease-free run takes from its policy, inherited by
    /// every lease-free descendant.
    serial_memory: Option<SerialMemoryBudget>,
}

#[derive(Default)]
struct WorkerActivity {
    current: AtomicUsize,
    peak: AtomicUsize,
}

#[derive(Default)]
struct PhysicalActivity {
    current_memory: AtomicU64,
    peak_memory: AtomicU64,
    discarded_work: AtomicU64,
}

pub(crate) struct MemoryActivityGuard(Arc<PhysicalActivity>, u64);

impl Drop for MemoryActivityGuard {
    fn drop(&mut self) {
        self.0.current_memory.fetch_sub(self.1, Ordering::AcqRel);
    }
}

pub(crate) struct WorkerActivityGuard(Arc<WorkerActivity>);

impl Drop for WorkerActivityGuard {
    fn drop(&mut self) {
        self.0.current.fetch_sub(1, Ordering::AcqRel);
    }
}

impl RunLimits {
    pub(crate) fn poll_request(&self) -> Result<(), KernelStop> {
        self.safe_point(&mut NestedStop::Clear)
    }
    pub(crate) fn work_stop_owner(&self) -> NestedStopOwner {
        self.ceiling_owner
    }
    pub(crate) fn has_parent() -> bool {
        ACTIVE_METER.with(|active| !active.borrow().is_empty())
    }

    pub(crate) fn has_leased_parent() -> bool {
        ACTIVE_METER.with(|active| {
            active
                .borrow()
                .last()
                .is_some_and(|parent| parent.borrow().limits.bound_to_lease)
        })
    }

    fn safe_point(&self, stop: &mut NestedStop) -> Result<(), KernelStop> {
        if self.tokens.iter().any(CancellationToken::is_cancelled) {
            stop.record(NestedStopOwner::Request);
            return Err(KernelStop::Cancelled);
        }
        if self
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            stop.record(NestedStopOwner::Request);
            return Err(KernelStop::DeadlineElapsed);
        }
        Ok(())
    }

    pub(crate) fn ceiling(&self) -> u64 {
        self.ceiling
    }

    pub(crate) fn force_serial(&self) -> bool {
        self.force_serial
    }

    pub(crate) fn serial_memory(&self) -> Option<&SerialMemoryBudget> {
        self.serial_memory.as_ref()
    }

    pub(crate) fn enter_physical_worker(&self, newly_active: bool) -> Option<WorkerActivityGuard> {
        if !newly_active {
            return None;
        }
        let active = self.worker_activity.current.fetch_add(1, Ordering::AcqRel) + 1;
        self.worker_activity
            .peak
            .fetch_max(active, Ordering::AcqRel);
        Some(WorkerActivityGuard(Arc::clone(&self.worker_activity)))
    }

    pub(crate) fn worker_high_watermark(&self) -> usize {
        self.worker_activity.peak.load(Ordering::Acquire)
    }

    pub(crate) fn enter_memory(&self, bytes: u64) -> MemoryActivityGuard {
        let current = self
            .physical_activity
            .current_memory
            .fetch_add(bytes, Ordering::AcqRel)
            + bytes;
        self.physical_activity
            .peak_memory
            .fetch_max(current, Ordering::AcqRel);
        MemoryActivityGuard(Arc::clone(&self.physical_activity), bytes)
    }

    pub(crate) fn peak_memory(&self) -> u64 {
        self.physical_activity.peak_memory.load(Ordering::Acquire)
    }

    pub(crate) fn add_discarded_work(&self, units: u64) {
        self.physical_activity
            .discarded_work
            .fetch_add(units, Ordering::AcqRel);
    }

    pub(crate) fn discarded_work(&self) -> u64 {
        self.physical_activity
            .discarded_work
            .load(Ordering::Acquire)
    }
}

struct KernelMeter {
    capture: Option<kernel_receipt::Capture>,
    limits: RunLimits,
    work: u64,
    span: u64,
    nested_stop: NestedStop,
    pending_request_refusal: bool,
    checkpoint_stop: Option<KernelStop>,
}

/// Worker-local checkpoint and cost context. Its fields are private so a
/// kernel cannot claim uncharged work or widen inherited limits.
pub struct KernelContext<'a, 'lease> {
    meter: Rc<RefCell<KernelMeter>>,
    _lease: PhantomData<&'a ExecutionResourceLease<'lease>>,
}

impl<'a, 'lease> KernelContext<'a, 'lease> {
    pub(crate) fn safe_point(&self) -> Result<(), KernelStop> {
        let mut meter = self.meter.borrow_mut();
        let KernelMeter {
            limits,
            nested_stop,
            ..
        } = &mut *meter;
        limits.safe_point(nested_stop)
    }
    pub(crate) fn new(limits: RunLimits) -> Self {
        Self {
            meter: Rc::new(RefCell::new(KernelMeter {
                capture: None,
                limits,
                work: 0,
                span: 0,
                nested_stop: NestedStop::Clear,
                pending_request_refusal: false,
                checkpoint_stop: None,
            })),
            _lease: PhantomData,
        }
    }

    pub(crate) fn enter(&self) -> KernelMeterGuard {
        ACTIVE_METER.with(|active| active.borrow_mut().push(Rc::clone(&self.meter)));
        KernelMeterGuard
    }

    pub(crate) fn cost(&self) -> (u64, u64) {
        let meter = self.meter.borrow();
        (meter.work, meter.span)
    }

    pub(crate) fn nested_stopped(&self) -> bool {
        self.meter.borrow().nested_stop.owner().is_some()
    }

    pub(crate) fn checkpoint_stop(&self) -> Option<KernelStop> {
        self.meter.borrow().checkpoint_stop
    }

    /// Account work completed by a domain-local checked meter after it stops.
    /// This does not authorize new work or clear the original stop; it keeps
    /// cancellation/deadline reports faithful to work already performed.
    pub fn account_completed_work(&mut self, units: u64) -> Result<(), KernelStop> {
        if let Some(capture) = &mut self.meter.borrow_mut().capture {
            capture.invalidate();
        }
        if units > 0 {
            crate::authority::consult_observation::charged_prefix(|| {
                ACTIVE_METER.with(|active| {
                    active
                        .borrow()
                        .iter()
                        .try_fold(units, |sum, meter| sum.checked_add(meter.borrow().work))
                        .unwrap_or(u64::MAX)
                })
            });
        }
        let mut meter = self.meter.borrow_mut();
        let result = (|| {
            let next = meter
                .work
                .checked_add(units)
                .ok_or(KernelStop::WorkCounterOverflow)?;
            let span = meter
                .span
                .checked_add(units)
                .ok_or(KernelStop::WorkCounterOverflow)?;
            if next > meter.limits.ceiling {
                return Err(KernelStop::WorkCeiling);
            }
            meter.work = next;
            meter.span = span;
            Ok(())
        })();
        if let Err(stop) = result {
            meter.checkpoint_stop.get_or_insert(stop);
            if stop == KernelStop::WorkCounterOverflow
                || meter.limits.ceiling_owner == NestedStopOwner::Request
            {
                meter.nested_stop.record(NestedStopOwner::Request);
            }
        }
        result
    }

    pub fn remaining_work(&self) -> u64 {
        let mut meter = self.meter.borrow_mut();
        if let Some(capture) = &mut meter.capture {
            capture.invalidate();
        }
        meter.limits.ceiling.saturating_sub(meter.work)
    }
}

pub(crate) struct KernelMeterGuard;

impl Drop for KernelMeterGuard {
    fn drop(&mut self) {
        ACTIVE_METER.with(|active| {
            active.borrow_mut().pop().expect("balanced kernel meter");
        });
    }
}

// The owned observation guard lends, but cannot replace, the active meter.
pub(super) fn visit_active_meter_for_test(
    mut visit: impl FnMut(Option<&mut KernelContext<'_, '_>>),
) {
    let meter = ACTIVE_METER.with(|stack| stack.borrow().last().cloned());
    match meter {
        Some(meter) => {
            let mut context = KernelContext {
                meter,
                _lease: PhantomData,
            };
            visit(Some(&mut context));
        }
        None => visit(None),
    }
}
