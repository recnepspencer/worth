//! Original Source preparation borrows the active SDK account, never a second allowance.
use super::*;
use worth_execution::{
    ExecutionRequest, ExecutionWorkLoanDenial, MapKernelContext, MapKernelStop, WorkCeilingDenial,
};
impl CustomPreparationBudget {
    pub(crate) fn source_stage(
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Self, WorkCeilingDenial> {
        let base = meter_metadata_bytes();
        let payer = execution.reserve_retained_memory(base)?;
        let budget = Self::with_control(
            PreparationControl::Stage(context.work_loan(execution)?),
            None,
            None,
        );
        if let Some(payer) = payer {
            *budget.0.memory.lock().unwrap_or_else(|e| e.into_inner()) =
                Some(PreparationMemory::Owned { payer, base });
        }
        Ok(budget)
    }
    pub(crate) fn request_stop(&self) -> Option<MapKernelStop> {
        *self
            .0
            .request_stop
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }
}
impl CustomInvariantWorkMeter {
    pub(super) fn loan_failed(&self, cause: ExecutionWorkLoanDenial) {
        match cause {
            ExecutionWorkLoanDenial::Request(stop) => self.record_native_stop(stop),
            ExecutionWorkLoanDenial::ClosedScope | ExecutionWorkLoanDenial::ForeignScope => {
                self.stop(CustomPreparationStop::UncheckedCustomKernel)
            }
        }
    }
    pub(crate) fn record_native_stop(&self, stop: MapKernelStop) {
        if let Some(leased) = &self.leased {
            leased
                .request_stop
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .get_or_insert(stop);
        }
        self.stop(CustomPreparationStop::Request(stop));
    }
    pub(crate) fn admits_native_context(&self, context: &MapKernelContext<'_, '_>) -> bool {
        if !context.is_current() {
            return false;
        }
        match self.leased.as_ref().map(|meter| &meter.status) {
            Some(PreparationControl::Stage(loan)) => loan.validate_context(context).is_ok(),
            _ => true,
        }
    }
}
