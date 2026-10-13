use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use worth_execution::ExecutionLeaseStatus;

use crate::execution::InvariantScratchAdmission;

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

enum CustomMemoryAdmission {
    Prequoted(Option<u64>),
    Dynamic(InvariantScratchAdmission),
}

struct LeasedCustomMeter {
    status: ExecutionLeaseStatus,
    work_ceiling: Option<u64>,
    consumed_work: AtomicU64,
    memory: CustomMemoryAdmission,
    claimed_memory: AtomicU64,
    stop: std::sync::Mutex<Option<CustomPreparationStop>>,
}

#[derive(Clone)]
pub(crate) struct CustomPreparationBudget(Arc<LeasedCustomMeter>);

impl CustomPreparationBudget {
    pub(crate) fn allocation_bytes() -> u64 {
        (std::mem::size_of::<LeasedCustomMeter>() + 2 * std::mem::size_of::<usize>()) as u64
    }

    pub(crate) fn evaluation_meter_allocation_bytes() -> u64 {
        arc_allocation_bytes::<AtomicU64>() + arc_allocation_bytes::<AtomicBool>()
    }

    pub(crate) fn new(
        status: ExecutionLeaseStatus,
        work_ceiling: Option<u64>,
        memory_ceiling: Option<u64>,
    ) -> Self {
        Self(Arc::new(LeasedCustomMeter {
            status,
            work_ceiling,
            consumed_work: AtomicU64::new(0),
            memory: CustomMemoryAdmission::Prequoted(memory_ceiling),
            claimed_memory: AtomicU64::new(0),
            stop: std::sync::Mutex::new(None),
        }))
    }

    pub(crate) fn with_scratch_admission(
        status: ExecutionLeaseStatus,
        work_ceiling: Option<u64>,
        admission: InvariantScratchAdmission,
    ) -> Self {
        Self(Arc::new(LeasedCustomMeter {
            status,
            work_ceiling,
            consumed_work: AtomicU64::new(0),
            memory: CustomMemoryAdmission::Dynamic(admission),
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
        self.0.stop_reason()
    }

    pub(crate) fn deny_unchecked(&self) {
        self.0
            .record_stop(CustomPreparationStop::UncheckedCustomKernel);
    }

    /// Candidate-local inspection is request work, not a rule's allowance.
    pub(crate) fn try_visit(&self, units: usize) -> bool {
        self.0
            .try_charge_work(u64::try_from(units).unwrap_or(u64::MAX))
    }

    pub(crate) fn try_plan_item(&self, retained_bytes: u64) -> bool {
        self.0.try_charge_work(1) && self.try_claim_bytes(retained_bytes)
    }

    pub(crate) fn try_claim_bytes(&self, bytes: u64) -> bool {
        self.0.try_claim_bytes(bytes)
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
    pub(crate) fn new(maximum: NonZeroU64) -> Self {
        Self {
            maximum: Some(maximum),
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
        let remaining = self
            .maximum
            .expect("only installed invariant meters enter evaluation")
            .get()
            .saturating_sub(self.consumed.load(Ordering::Relaxed));
        let meter = Self::new(NonZeroU64::new(remaining.max(1)).unwrap()).with_lease(budget);
        if remaining == 0 {
            meter.stop(CustomPreparationStop::WorkExhausted);
        }
        meter
    }

    fn stop(&self, reason: CustomPreparationStop) {
        if let Some(leased) = &self.leased {
            leased.record_stop(reason);
        }
        self.mark_exceeded();
    }

    pub(crate) fn leased_stop(&self) -> Option<CustomPreparationStop> {
        self.leased.as_ref().and_then(|leased| leased.stop_reason())
    }

    pub(crate) fn try_claim_bytes(&self, bytes: u64) -> bool {
        if self
            .leased
            .as_ref()
            .is_none_or(|leased| leased.try_claim_bytes(bytes))
        {
            return true;
        }
        self.mark_exceeded();
        false
    }

    fn check_claim_peak(&self, bytes: u64) -> bool {
        if self
            .leased
            .as_ref()
            .is_none_or(|leased| leased.check_claim_peak(bytes))
        {
            return true;
        }
        self.mark_exceeded();
        false
    }

    fn safe_point(&self) -> bool {
        if self
            .leased
            .as_ref()
            .is_none_or(|leased| leased.safe_point())
        {
            return true;
        }
        self.mark_exceeded();
        false
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
        if self
            .leased
            .as_ref()
            .is_some_and(|leased| !leased.try_charge_work(units))
        {
            self.mark_exceeded();
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

impl LeasedCustomMeter {
    fn record_stop(&self, reason: CustomPreparationStop) {
        self.stop
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get_or_insert(reason);
    }

    fn stop_reason(&self) -> Option<CustomPreparationStop> {
        *self.stop.lock().unwrap_or_else(|error| error.into_inner())
    }

    fn safe_point(&self) -> bool {
        if self.status.is_cancelled() {
            self.record_stop(CustomPreparationStop::Cancelled);
            return false;
        }
        if self.status.deadline_elapsed() {
            self.record_stop(CustomPreparationStop::DeadlineElapsed);
            return false;
        }
        self.stop_reason().is_none()
    }

    fn try_charge_work(&self, units: u64) -> bool {
        if !self.safe_point() {
            return false;
        }
        if self
            .consumed_work
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current
                    .checked_add(units)
                    .filter(|next| self.work_ceiling.is_none_or(|ceiling| *next <= ceiling))
            })
            .is_err()
        {
            self.record_stop(CustomPreparationStop::WorkExhausted);
            return false;
        }
        true
    }

    fn try_claim_bytes(&self, bytes: u64) -> bool {
        if !self.safe_point() {
            return false;
        }
        if let CustomMemoryAdmission::Dynamic(admission) = &self.memory {
            if admission.claim(bytes).is_err() {
                self.record_stop(CustomPreparationStop::MemoryExhausted);
                return false;
            }
        }
        if self
            .claimed_memory
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current
                    .checked_add(bytes)
                    .filter(|next| match &self.memory {
                        CustomMemoryAdmission::Prequoted(ceiling) => {
                            ceiling.is_none_or(|ceiling| *next <= ceiling)
                        }
                        CustomMemoryAdmission::Dynamic(_) => true,
                    })
            })
            .is_err()
        {
            self.record_stop(CustomPreparationStop::MemoryExhausted);
            return false;
        }
        true
    }

    fn check_claim_peak(&self, bytes: u64) -> bool {
        if !self.safe_point() {
            return false;
        }
        let admitted = match &self.memory {
            CustomMemoryAdmission::Prequoted(ceiling) => self
                .claimed_memory
                .load(Ordering::Relaxed)
                .checked_add(bytes)
                .is_some_and(|peak| ceiling.is_none_or(|ceiling| peak <= ceiling)),
            CustomMemoryAdmission::Dynamic(admission) => admission.check_peak(bytes).is_ok(),
        };
        if !admitted {
            self.record_stop(CustomPreparationStop::MemoryExhausted);
        }
        admitted
    }
}

// Include both Arc counters and the backing allocation's alignment padding.
fn arc_allocation_bytes<T>() -> u64 {
    let (layout, _) = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .expect("fixed meter metadata fits an allocation layout");
    layout.pad_to_align().size() as u64
}

#[cfg(test)]
mod allocation_tests;
