use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use worth_execution::ExecutionLeaseStatus;

// Fixed-size ID, BTree node, and wrapper retention for materializing
// structural visits. Borrowed reads and authored checkpoints charge work only;
// variable-size keys and aspect values are claimed separately.
const STRUCTURAL_UNIT_BYTES: u64 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CustomPreparationStop {
    Cancelled,
    DeadlineElapsed,
    WorkExhausted,
    MemoryExhausted,
    UncheckedCustomKernel,
}

struct LeasedCustomMeter {
    status: ExecutionLeaseStatus,
    work_ceiling: Option<u64>,
    consumed_work: AtomicU64,
    memory_ceiling: Option<u64>,
    claimed_memory: AtomicU64,
    stop: std::sync::Mutex<Option<CustomPreparationStop>>,
}

#[derive(Clone)]
pub(crate) struct CustomPreparationBudget(Arc<LeasedCustomMeter>);

impl CustomPreparationBudget {
    pub(crate) fn new(
        status: ExecutionLeaseStatus,
        work_ceiling: Option<u64>,
        memory_ceiling: Option<u64>,
    ) -> Self {
        Self(Arc::new(LeasedCustomMeter {
            status,
            work_ceiling,
            consumed_work: AtomicU64::new(0),
            memory_ceiling,
            claimed_memory: AtomicU64::new(0),
            stop: std::sync::Mutex::new(None),
        }))
    }

    pub(crate) fn charged_work(&self) -> u64 {
        self.0.consumed_work.load(Ordering::Relaxed)
    }

    pub(crate) fn claimed_memory(&self) -> u64 {
        self.0.claimed_memory.load(Ordering::Relaxed)
    }

    pub(crate) fn stop(&self) -> Option<CustomPreparationStop> {
        *self
            .0
            .stop
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    pub(crate) fn deny_unchecked(&self) {
        let mut stop = self
            .0
            .stop
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        stop.get_or_insert(CustomPreparationStop::UncheckedCustomKernel);
    }

    /// Candidate-local inspection is request work, not a rule's allowance.
    pub(crate) fn try_visit(&self, units: usize) -> bool {
        let meter = CustomInvariantWorkMeter {
            maximum: None,
            consumed: Arc::new(AtomicU64::new(0)),
            exhausted: Arc::new(AtomicBool::new(false)),
            leased: Some(Arc::clone(&self.0)),
        };
        meter.try_charge_work(u64::try_from(units).unwrap_or(u64::MAX))
    }

    pub(crate) fn try_plan_item(&self, retained_bytes: u64) -> bool {
        let meter = CustomInvariantWorkMeter {
            maximum: None,
            consumed: Arc::new(AtomicU64::new(0)),
            exhausted: Arc::new(AtomicBool::new(false)),
            leased: Some(Arc::clone(&self.0)),
        };
        meter.try_charge_work(1) && meter.try_claim_bytes(retained_bytes)
    }
}

#[derive(Clone)]
pub(crate) struct CustomInvariantWorkMeter {
    maximum: Option<NonZeroU64>,
    consumed: Arc<AtomicU64>,
    exhausted: Arc<AtomicBool>,
    leased: Option<Arc<LeasedCustomMeter>>,
}

impl CustomInvariantWorkMeter {
    pub(crate) fn new(maximum: Option<NonZeroU64>) -> Self {
        Self {
            maximum,
            consumed: Arc::new(AtomicU64::new(0)),
            exhausted: Arc::new(AtomicBool::new(false)),
            leased: None,
        }
    }

    pub(crate) fn with_lease(mut self, budget: &CustomPreparationBudget) -> Self {
        self.leased = Some(Arc::clone(&budget.0));
        self
    }

    pub(crate) fn for_evaluation(&self, budget: &CustomPreparationBudget) -> Self {
        let Some(maximum) = self.maximum else {
            return Self::new(None).with_lease(budget);
        };
        let remaining = maximum
            .get()
            .saturating_sub(self.consumed.load(Ordering::Relaxed));
        let Some(remaining) = NonZeroU64::new(remaining) else {
            let meter = Self::new(Some(maximum)).with_lease(budget);
            meter.stop(CustomPreparationStop::WorkExhausted);
            return meter;
        };
        Self::new(Some(remaining)).with_lease(budget)
    }

    fn stop(&self, reason: CustomPreparationStop) {
        if let Some(leased) = &self.leased {
            let mut stop = leased
                .stop
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            stop.get_or_insert(reason);
        }
        self.mark_exceeded();
    }

    pub(crate) fn leased_stop(&self) -> Option<CustomPreparationStop> {
        self.leased.as_ref().and_then(|leased| {
            *leased
                .stop
                .lock()
                .unwrap_or_else(|error| error.into_inner())
        })
    }

    pub(crate) fn try_claim_bytes(&self, bytes: u64) -> bool {
        let Some(leased) = &self.leased else {
            return true;
        };
        if !self.safe_point() {
            return false;
        }
        if leased
            .claimed_memory
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current
                    .checked_add(bytes)
                    .filter(|next| leased.memory_ceiling.is_none_or(|ceiling| *next <= ceiling))
            })
            .is_err()
        {
            self.stop(CustomPreparationStop::MemoryExhausted);
            return false;
        }
        true
    }

    fn check_claim_peak(&self, bytes: u64) -> bool {
        let Some(leased) = &self.leased else {
            return true;
        };
        self.safe_point()
            && leased
                .claimed_memory
                .load(Ordering::Relaxed)
                .checked_add(bytes)
                .is_some_and(|peak| leased.memory_ceiling.is_none_or(|ceiling| peak <= ceiling))
    }

    fn safe_point(&self) -> bool {
        let Some(leased) = &self.leased else {
            return true;
        };
        if leased.status.is_cancelled() {
            self.stop(CustomPreparationStop::Cancelled);
            return false;
        }
        if leased.status.deadline_elapsed() {
            self.stop(CustomPreparationStop::DeadlineElapsed);
            return false;
        }
        self.leased_stop().is_none()
    }
    pub(crate) fn try_charge(&self, units: usize) -> bool {
        if self.exceeded() || !self.safe_point() {
            return false;
        }
        let Ok(units) = u64::try_from(units) else {
            self.stop(CustomPreparationStop::WorkExhausted);
            return false;
        };
        if !self.try_claim_bytes(units.saturating_mul(STRUCTURAL_UNIT_BYTES)) {
            return false;
        }
        self.try_charge_work(units)
    }

    // Read-only planning visits have work but no allocation. Every retained
    // plan payload is claimed explicitly by try_plan_item's byte declaration.
    pub(super) fn try_charge_work(&self, units: u64) -> bool {
        if self.exceeded() || !self.safe_point() {
            return false;
        }
        if let Some(leased) = &self.leased
            && leased
                .consumed_work
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                    current
                        .checked_add(units)
                        .filter(|next| leased.work_ceiling.is_none_or(|ceiling| *next <= ceiling))
                })
                .is_err()
        {
            self.stop(CustomPreparationStop::WorkExhausted);
            return false;
        }
        if self
            .consumed
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current
                    .checked_add(units)
                    .filter(|next| self.maximum.is_none_or(|maximum| *next <= maximum.get()))
            })
            .is_err()
        {
            self.stop(CustomPreparationStop::WorkExhausted);
            return false;
        }
        true
    }
    /// Exact charged units; diagnostic provenance keeps its nonzero minimum.
    pub(crate) fn charged_work(&self) -> u64 {
        self.consumed.load(Ordering::Relaxed)
    }

    pub(crate) fn consumed(&self) -> NonZeroU64 {
        NonZeroU64::new(self.charged_work().max(1)).unwrap()
    }
    pub(crate) fn exceeded(&self) -> bool {
        self.exhausted.load(Ordering::Relaxed)
    }
    pub(crate) fn mark_exceeded(&self) {
        self.exhausted.store(true, Ordering::Relaxed);
    }
}

impl crate::validation::data::CustomInvariantLeaseBudget for CustomInvariantWorkMeter {
    fn checkpoint(&self, units: u64) -> bool {
        self.try_charge_work(units)
    }

    fn claim_result(&self, bytes: u64) -> bool {
        self.try_claim_bytes(bytes)
    }

    fn claim_scratch(&self, bytes: u64) -> bool {
        self.try_claim_bytes(bytes)
    }

    fn check_scratch_peak(&self, bytes: u64) -> bool {
        self.check_claim_peak(bytes)
    }
}

#[cfg(test)]
mod allocation_tests;
