//! Borrowed consumption of the structural reader's original request and payer.
use super::{
    CustomInvariantWorkMeter, CustomPreparationStop, StructuralReadError, StructuralWalkDenialKind,
};

/// Read work and retention admission on the original custom-invariant meter.
/// This handle creates no allowance and grants no graph or mutation authority.
pub struct CustomInvariantReadBudget<'borrow> {
    execution: worth_execution::ExecutionRequest<'borrow, 'borrow>,
    work: &'borrow CustomInvariantWorkMeter,
}

impl<'borrow> CustomInvariantReadBudget<'borrow> {
    pub(crate) fn debit_native_rule_work(
        &self,
        units: u64,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        use crate::execution::RelationalExecutionDenialCause as Cause;
        self.execution.check_live().map_err(Cause::from)?;
        if self.work.try_debit_native_work(units) {
            return Ok(());
        }
        Err(match self.work.leased_stop() {
            Some(CustomPreparationStop::Cancelled) => Cause::Cancelled,
            Some(CustomPreparationStop::DeadlineElapsed) => Cause::DeadlineElapsed,
            Some(CustomPreparationStop::MemoryAdmission(denial)) => {
                crate::execution::denial::lease_denial(
                    worth_execution::LeaseDenial::MemoryExhausted(denial),
                )
            }
            Some(CustomPreparationStop::MemoryExhausted) => Cause::ScratchCapacityExceeded,
            Some(CustomPreparationStop::UncheckedCustomKernel) => Cause::UncheckedCustomKernel,
            _ => Cause::WorkExhausted,
        })
    }

    pub(super) fn new(
        execution: worth_execution::ExecutionRequest<'borrow, 'borrow>,
        work: &'borrow CustomInvariantWorkMeter,
    ) -> Self {
        Self { execution, work }
    }

    pub fn check_live(&self) -> Result<(), StructuralWalkDenialKind> {
        super::interruption::check_live(self.execution)
            .map_err(StructuralWalkDenialKind::Interrupted)
    }

    pub fn checkpoint(&self, units: u64) -> Result<(), StructuralWalkDenialKind> {
        self.check_live()?;
        self.work
            .try_charge_work(units)
            .then_some(())
            .ok_or_else(|| self.refusal(StructuralReadError::WorkBudgetExceeded))
    }

    /// Admit backing before allocating it. Retention remains on the existing
    /// native payer; an escaping complete walk retains that payer itself.
    pub fn claim_retained(&self, bytes: u64) -> Result<(), StructuralWalkDenialKind> {
        self.checkpoint(0)?;
        self.work
            .try_claim_bytes(bytes)
            .then_some(())
            .ok_or_else(|| self.refusal(StructuralReadError::WorkBudgetExceeded))
    }

    pub(crate) fn refusal(&self, error: StructuralReadError) -> StructuralWalkDenialKind {
        use super::CustomInvariantInterruption;
        match self.work.leased_stop() {
            Some(CustomPreparationStop::Cancelled) => {
                StructuralWalkDenialKind::Interrupted(CustomInvariantInterruption::Cancelled)
            }
            Some(CustomPreparationStop::DeadlineElapsed) => {
                StructuralWalkDenialKind::Interrupted(CustomInvariantInterruption::DeadlineElapsed)
            }
            Some(CustomPreparationStop::MemoryExhausted) => {
                StructuralWalkDenialKind::MemoryBudgetExceeded
            }
            Some(CustomPreparationStop::MemoryAdmission(cause)) => {
                StructuralWalkDenialKind::MemoryAdmission(cause)
            }
            _ => StructuralWalkDenialKind::Read(error),
        }
    }
}
