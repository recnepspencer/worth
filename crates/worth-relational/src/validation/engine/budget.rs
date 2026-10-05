use std::cell::RefCell;

use worth_execution::{ExecutionLeaseStatus, MapKernelFailure};

use crate::execution::{PacketBudgetDenial, PacketKernelContext};
use crate::validation::custom_rule::{CustomPreparationBudget, CustomPreparationStop};

pub(crate) type InvariantBudgetStop = MapKernelFailure<PacketBudgetDenial>;

/// Interior mutability permits existing rule evaluators to inspect an immutable
/// invariant context while recording a typed execution stop at each candidate.
pub(crate) trait InvariantBudget {
    fn checkpoint(&self, units: u64) -> bool;
    fn claim_result(&self, bytes: u64) -> bool;
    fn claim_scratch(&self, bytes: u64) -> bool;
    fn check_scratch_peak(&self, bytes: u64) -> bool;
    fn ensure_result_total(&self, bytes: u64) -> bool;
    fn check_result_peak(&self, bytes: u64) -> bool;
    fn custom_meter(&self) -> &dyn crate::validation::data::CustomInvariantLeaseBudget;
    fn deny_unchecked_custom(&self);
    fn custom_structural_budget(&self) -> CustomPreparationBudget;
    fn settle_custom_structural(&self, budget: &CustomPreparationBudget) -> bool;
}

pub(crate) struct LeasedInvariantBudget<'a, 'b, 'c, 'd> {
    context: RefCell<&'a mut PacketKernelContext<'b, 'c, 'd>>,
    failure: RefCell<Option<InvariantBudgetStop>>,
    status: ExecutionLeaseStatus,
}

impl<'a, 'b, 'c, 'd> LeasedInvariantBudget<'a, 'b, 'c, 'd> {
    pub(crate) fn new(
        context: &'a mut PacketKernelContext<'b, 'c, 'd>,
        status: ExecutionLeaseStatus,
    ) -> Self {
        Self {
            context: RefCell::new(context),
            failure: RefCell::new(None),
            status,
        }
    }

    pub(crate) fn take_failure(&self) -> Option<InvariantBudgetStop> {
        self.failure.borrow_mut().take()
    }

    fn attempt(
        &self,
        operation: impl FnOnce(&mut PacketKernelContext<'_, '_, '_>) -> Result<(), InvariantBudgetStop>,
    ) -> bool {
        if self.failure.borrow().is_some() {
            return false;
        }
        match operation(&mut self.context.borrow_mut()) {
            Ok(()) => true,
            Err(error) => {
                *self.failure.borrow_mut() = Some(error);
                false
            }
        }
    }
}

impl InvariantBudget for LeasedInvariantBudget<'_, '_, '_, '_> {
    fn checkpoint(&self, units: u64) -> bool {
        self.attempt(|context| context.checkpoint(units))
    }

    fn claim_result(&self, bytes: u64) -> bool {
        self.attempt(|context| context.claim_result(bytes))
    }
    fn claim_scratch(&self, bytes: u64) -> bool {
        self.attempt(|context| context.claim_scratch(bytes))
    }

    fn check_scratch_peak(&self, bytes: u64) -> bool {
        self.attempt(|context| context.check_scratch_peak(bytes))
    }

    fn ensure_result_total(&self, bytes: u64) -> bool {
        self.attempt(|context| context.ensure_result_total(bytes))
    }
    fn check_result_peak(&self, bytes: u64) -> bool {
        self.attempt(|context| context.check_result_peak(bytes))
    }

    fn custom_meter(&self) -> &dyn crate::validation::data::CustomInvariantLeaseBudget {
        self
    }

    fn deny_unchecked_custom(&self) {
        *self.failure.borrow_mut() = Some(MapKernelFailure::Domain(
            PacketBudgetDenial::UncheckedCustomKernel,
        ));
    }

    fn custom_structural_budget(&self) -> CustomPreparationBudget {
        let context = self.context.borrow();
        CustomPreparationBudget::new(
            self.status.clone(),
            context.remaining_work(),
            context.remaining_scratch(),
        )
    }

    fn settle_custom_structural(&self, budget: &CustomPreparationBudget) -> bool {
        let mut context = self.context.borrow_mut();
        // Flush measured view work even when the view observed cancellation.
        let work_result = context.account_completed_work(budget.charged_work());
        let memory_result = context.claim_scratch(budget.claimed_memory());
        let stop = budget.stop();
        let mut failure = self.failure.borrow_mut();
        if failure.is_none() {
            *failure = work_result
                .err()
                .or_else(|| memory_result.err())
                .or_else(|| {
                    stop.map(|stop| match stop {
                        CustomPreparationStop::Cancelled => {
                            MapKernelFailure::Stop(worth_execution::MapKernelStop::Cancelled)
                        }
                        CustomPreparationStop::DeadlineElapsed => {
                            MapKernelFailure::Stop(worth_execution::MapKernelStop::DeadlineElapsed)
                        }
                        CustomPreparationStop::WorkExhausted => {
                            MapKernelFailure::Stop(worth_execution::MapKernelStop::WorkCeiling)
                        }
                        CustomPreparationStop::MemoryExhausted => {
                            MapKernelFailure::Domain(PacketBudgetDenial::ScratchCapacityExceeded)
                        }
                        CustomPreparationStop::UncheckedCustomKernel => {
                            MapKernelFailure::Domain(PacketBudgetDenial::UncheckedCustomKernel)
                        }
                    })
                });
        }
        failure.is_none()
    }
}

impl crate::validation::data::CustomInvariantLeaseBudget for LeasedInvariantBudget<'_, '_, '_, '_> {
    fn checkpoint(&self, units: u64) -> bool {
        InvariantBudget::checkpoint(self, units)
    }
    fn claim_result(&self, bytes: u64) -> bool {
        InvariantBudget::claim_result(self, bytes)
    }
    fn claim_scratch(&self, bytes: u64) -> bool {
        InvariantBudget::claim_scratch(self, bytes)
    }
    fn check_scratch_peak(&self, bytes: u64) -> bool {
        InvariantBudget::check_scratch_peak(self, bytes)
    }
}
