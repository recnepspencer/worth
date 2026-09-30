use std::{
    cell::RefCell,
    marker::PhantomData,
    mem::size_of,
    rc::Rc,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use worth_foundational::ExecutionReport;

use crate::authority::{CancellationToken, ExecutionResourceLease};

thread_local! {
    static ACTIVE_METER: RefCell<Vec<Rc<RefCell<KernelMeter>>>> = const { RefCell::new(Vec::new()) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelStop {
    Cancelled,
    DeadlineElapsed,
    WorkCounterOverflow,
    WorkCeiling,
    NestedStopped,
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
    force_serial: bool,
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
    pub(crate) fn for_run(lease: Option<&ExecutionResourceLease<'_>>, serial: bool) -> Self {
        let parent = ACTIVE_METER.with(|active| active.borrow().last().cloned());
        let (tokens, deadline, ceiling, worker_activity, physical_activity, inherited_serial) =
            if let Some(parent) = parent {
                let parent = parent.borrow();
                (
                    parent.limits.tokens.clone(),
                    parent.limits.deadline,
                    parent.limits.ceiling.saturating_sub(parent.work),
                    Arc::clone(&parent.limits.worker_activity),
                    Arc::clone(&parent.limits.physical_activity),
                    parent.limits.force_serial,
                )
            } else {
                (
                    Vec::new(),
                    None,
                    u64::MAX,
                    Arc::new(WorkerActivity::default()),
                    Arc::new(PhysicalActivity::default()),
                    false,
                )
            };
        let mut limits = Self {
            tokens,
            deadline,
            ceiling,
            worker_activity,
            physical_activity,
            force_serial: serial || inherited_serial,
        };
        if let Some(lease) = lease {
            limits.tokens.extend(lease.cancellation_lineage());
            limits.deadline = match (limits.deadline, lease.deadline()) {
                (Some(parent), Some(child)) => Some(parent.min(child)),
                (Some(parent), None) => Some(parent),
                (None, child) => child,
            };
            limits.ceiling = limits.ceiling.min(lease.policy().budget().work_ceiling());
        }
        limits
    }

    pub(crate) fn has_parent() -> bool {
        ACTIVE_METER.with(|active| !active.borrow().is_empty())
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

    pub(crate) fn framework_context_bytes(
        &self,
        partitions: usize,
        max_workers: usize,
    ) -> Option<u64> {
        let token_bytes = self
            .tokens
            .len()
            .checked_mul(size_of::<CancellationToken>())?;
        let per_partition = size_of::<KernelMeter>()
            .checked_add(2 * size_of::<usize>())? // Rc allocation header
            .checked_add(token_bytes)? // cloned checkpoint lineage
            .checked_add(size_of::<(usize, u64)>())?; // worker context stack
        let partition_bytes = partitions.checked_mul(per_partition)?;
        let worker_guards =
            max_workers.checked_mul(size_of::<crate::authority::ResourceReservation>())?;
        let fixed = size_of::<WorkerActivity>()
            .checked_add(2 * size_of::<usize>())? // Arc allocation header
            .checked_add(token_bytes)?
            .checked_add(worker_guards)?;
        u64::try_from(partition_bytes.checked_add(fixed)?).ok()
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
}

pub(crate) struct KernelMeterGuard;

impl Drop for KernelMeterGuard {
    fn drop(&mut self) {
        ACTIVE_METER.with(|active| {
            active.borrow_mut().pop().expect("balanced kernel meter");
        });
    }
}

/// Add a joined nested computation to the invoking partition, never to a
/// global completion-order counter. The enclosing partition's canonical
/// identity determines whether this cost is ultimately charged.
pub(crate) fn record_nested(report: ExecutionReport, stopped: bool) {
    ACTIVE_METER.with(|active| {
        if let Some(parent) = active.borrow().last() {
            let mut parent = parent.borrow_mut();
            parent.work = match parent.work.checked_add(report.charged_work()) {
                Some(work) => work,
                None => {
                    parent.nested_stopped = true;
                    u64::MAX
                }
            };
            parent.span = match parent.span.checked_add(report.charged_span()) {
                Some(span) => span,
                None => {
                    parent.nested_stopped = true;
                    u64::MAX
                }
            };
            parent.nested_stopped |= stopped;
            if parent.work > parent.limits.ceiling {
                parent.nested_stopped = true;
            }
        }
    });
}

pub(crate) fn enter_retained_memory(bytes: u64) -> Option<MemoryActivityGuard> {
    ACTIVE_METER.with(|active| {
        let parent = active.borrow().last().cloned()?;
        let guard = parent.borrow().limits.enter_memory(bytes);
        Some(guard)
    })
}
