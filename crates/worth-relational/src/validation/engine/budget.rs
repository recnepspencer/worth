use std::cell::RefCell;

use worth_execution::{ExecutionLeaseStatus, MapKernelFailure};

use crate::execution::{PacketBudgetDenial, PacketKernelContext};

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
    fn remaining_scratch(&self) -> Option<u64>;
    fn custom_structural_budget(
        &self,
    ) -> Option<crate::validation::custom_rule::CustomPreparationBudget>;
    fn settle_custom_structural(
        &self,
        budget: &crate::validation::custom_rule::CustomPreparationBudget,
    ) -> bool;
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
    fn remaining_scratch(&self) -> Option<u64> {
        self.context.borrow().remaining_scratch()
    }
    fn custom_structural_budget(
        &self,
    ) -> Option<crate::validation::custom_rule::CustomPreparationBudget> {
        use crate::validation::custom_rule::CustomPreparationBudget;
        let context = self.context.borrow();
        let result = match context.structural_memory() {
            Some(payer) => CustomPreparationBudget::packet_tracked(
                self.status.clone(),
                context.remaining_work(),
                payer,
            ),
            None => Ok(CustomPreparationBudget::new(
                self.status.clone(),
                context.remaining_work(),
                context.remaining_scratch(),
            )),
        };
        match result {
            Ok(budget) => Some(budget),
            Err(denial) => {
                *self.failure.borrow_mut() = Some(MapKernelFailure::Domain(
                    PacketBudgetDenial::ReadRequest(crate::execution::denial::lease_denial(
                        worth_execution::LeaseDenial::MemoryExhausted(denial),
                    )),
                ));
                None
            }
        }
    }
    fn settle_custom_structural(
        &self,
        budget: &crate::validation::custom_rule::CustomPreparationBudget,
    ) -> bool {
        self.attempt(|context| {
            context.account_completed_work(budget.charged_work())?;
            if let Some(stop) = budget.stop() {
                return Err(structural_stop(stop));
            }
            if context.structural_memory().is_none() {
                context.claim_scratch(budget.claimed_memory())?;
            }
            Ok(())
        })
    }

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

fn structural_stop(
    stop: crate::validation::custom_rule::CustomPreparationStop,
) -> InvariantBudgetStop {
    use crate::validation::custom_rule::CustomPreparationStop as Stop;
    use worth_execution::MapKernelStop;
    match stop {
        Stop::Cancelled => MapKernelFailure::Stop(MapKernelStop::Cancelled),
        Stop::DeadlineElapsed => MapKernelFailure::Stop(MapKernelStop::DeadlineElapsed),
        Stop::WorkExhausted => MapKernelFailure::Stop(MapKernelStop::WorkCeiling),
        Stop::MemoryExhausted => {
            MapKernelFailure::Domain(PacketBudgetDenial::ScratchCapacityExceeded)
        }
        Stop::UncheckedCustomKernel => {
            MapKernelFailure::Domain(PacketBudgetDenial::UncheckedCustomKernel)
        }
        Stop::MemoryAdmission(denial) => MapKernelFailure::Domain(PacketBudgetDenial::ReadRequest(
            crate::execution::denial::lease_denial(worth_execution::LeaseDenial::MemoryExhausted(
                denial,
            )),
        )),
    }
}
