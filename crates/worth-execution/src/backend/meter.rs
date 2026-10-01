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
    authority::{CancellationToken, ExecutionResourceLease},
    report::ChargedBytes,
};

thread_local! {
    static ACTIVE_METER: RefCell<Vec<Rc<RefCell<KernelMeter>>>> = const { RefCell::new(Vec::new()) };
}

mod activity;
mod framework_bytes;
mod nested;
mod prepared_context;
mod run_context;
pub(crate) use activity::{enter_certification_activity, enter_retained_memory, has_active_kernel};
pub(crate) use nested::record_nested;
pub(crate) use prepared_context::PreparedMeterContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelStop {
    Cancelled,
    DeadlineElapsed,
    WorkCounterOverflow,
    WorkCeiling,
    NestedStopped,
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
    worker_activity: Arc<WorkerActivity>,
    physical_activity: Arc<PhysicalActivity>,
    physical_cell_charged: bool,
    force_serial: bool,
    bound_to_lease: bool,
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

    pub(crate) fn safe_point(&self) -> Result<(), KernelStop> {
        if self.tokens.iter().any(CancellationToken::is_cancelled) {
            return Err(KernelStop::Cancelled);
        }
        if self
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
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
    limits: RunLimits,
    work: u64,
    span: u64,
    nested_stopped: bool,
    checkpoint_stop: Option<KernelStop>,
}

/// Worker-local checkpoint and cost context. Its fields are private so a
/// kernel cannot claim uncharged work or widen inherited limits.
pub struct KernelContext<'a, 'lease> {
    meter: Rc<RefCell<KernelMeter>>,
    _lease: PhantomData<&'a ExecutionResourceLease<'lease>>,
}

impl<'a, 'lease> KernelContext<'a, 'lease> {
    pub(crate) fn new(limits: RunLimits) -> Self {
        Self {
            meter: Rc::new(RefCell::new(KernelMeter {
                limits,
                work: 0,
                span: 0,
                nested_stopped: false,
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

    /// Replace the sequential checkpoint span of a completed stage with its
    /// proven dependency span. Work remains the exact checkpoint count.
    pub(crate) fn apply_structural_span(
        &mut self,
        before: (u64, u64),
        stage_work: u64,
        stage_span: u64,
    ) -> bool {
        let mut meter = self.meter.borrow_mut();
        let Some(expected_work) = before.0.checked_add(stage_work) else {
            return false;
        };
        let Some(expected_span) = before.1.checked_add(stage_work) else {
            return false;
        };
        let Some(next_span) = before.1.checked_add(stage_span) else {
            return false;
        };
        if meter.work != expected_work || meter.span != expected_span || stage_span > stage_work {
            return false;
        }
        meter.span = next_span;
        true
    }

    pub(crate) fn nested_stopped(&self) -> bool {
        self.meter.borrow().nested_stopped
    }

    pub(crate) fn checkpoint_stop(&self) -> Option<KernelStop> {
        self.meter.borrow().checkpoint_stop
    }

    /// Kernel checkpoint enforces the local remaining ceiling before work is
    /// done, while canonical settlement determines the accepted prefix.
    pub fn checkpoint(&mut self, units: u64) -> Result<(), KernelStop> {
        let mut meter = self.meter.borrow_mut();
        if let Some(stop) = meter.checkpoint_stop {
            return Err(stop);
        }
        let result = (|| {
            meter.limits.safe_point()?;
            if meter.nested_stopped {
                return Err(KernelStop::NestedStopped);
            }
            let next = meter
                .work
                .checked_add(units)
                .ok_or(KernelStop::WorkCounterOverflow)?;
            if next > meter.limits.ceiling {
                return Err(KernelStop::WorkCeiling);
            }
            let span = meter
                .span
                .checked_add(units)
                .ok_or(KernelStop::WorkCounterOverflow)?;
            meter.work = next;
            meter.span = span;
            Ok(())
        })();
        if let Err(stop) = result {
            meter.checkpoint_stop = Some(stop);
        }
        result
    }

    /// Account work completed by a domain-local checked meter after it stops.
    /// This does not authorize new work or clear the original stop; it keeps
    /// cancellation/deadline reports faithful to work already performed.
    pub fn account_completed_work(&mut self, units: u64) -> Result<(), KernelStop> {
        let mut meter = self.meter.borrow_mut();
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
    }

    pub fn remaining_work(&self) -> u64 {
        let meter = self.meter.borrow();
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
