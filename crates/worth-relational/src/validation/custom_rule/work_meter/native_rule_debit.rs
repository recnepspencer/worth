//! A borrowed debit against the original rule and request accounts.
use std::num::NonZeroU64;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};
use worth_execution::{ExecutionRequest, ExecutionWorkLoanDenial, MapKernelContext, MapKernelStop};

use crate::validation::custom_rule::interruption;
use crate::validation::data::{
    structural_read::StructuralReadError, structural_walk_denial::StructuralWalkDenialKind,
};

pub(crate) enum PreparationControl {
    Lease(worth_execution::ExecutionLeaseStatus),
    Stage(worth_execution::ExecutionWorkLoan),
}
impl PreparationControl {
    pub(crate) fn is_cancelled(&self) -> bool {
        match self {
            Self::Lease(status) => status.is_cancelled(),
            Self::Stage(_) => false,
        }
    }
    pub(crate) fn deadline_elapsed(&self) -> bool {
        match self {
            Self::Lease(status) => status.deadline_elapsed(),
            Self::Stage(_) => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CustomPreparationStop {
    Cancelled,
    DeadlineElapsed,
    WorkExhausted,
    MemoryExhausted,
    MemoryAdmission(worth_execution::MemoryLimitDenial),
    UncheckedCustomKernel,
    Request(worth_execution::MapKernelStop),
}

/// The rule's own allowance and the SDK's original request remain distinct.
#[derive(Debug)]
pub enum CustomInvariantNativeCheckpointDenial {
    Source(StructuralWalkDenialKind),
    Request(worth_execution::MapKernelStop),
}

/// Borrowed stop state from the same funded request as the rule counters.
pub(super) struct BorrowedPreparationStops<'a> {
    pub(super) control: &'a PreparationControl,
    pub(super) first_stop: &'a Mutex<Option<CustomPreparationStop>>,
    pub(super) request_stop: &'a Mutex<Option<MapKernelStop>>,
}

/// Exclusive to the request's existing rule meter. This borrow creates no allowance.
pub(crate) struct NativeRuleDebit<'a> {
    maximum: Option<NonZeroU64>,
    consumed: &'a AtomicU64,
    exhausted: &'a AtomicBool,
    leased: Option<BorrowedPreparationStops<'a>>,
}

impl<'a> NativeRuleDebit<'a> {
    pub(super) fn borrow(
        maximum: Option<NonZeroU64>,
        consumed: &'a AtomicU64,
        exhausted: &'a AtomicBool,
        leased: Option<BorrowedPreparationStops<'a>>,
    ) -> Self {
        Self {
            maximum,
            consumed,
            exhausted,
            leased,
        }
    }

    fn exceeded(&self) -> bool {
        self.exhausted.load(Ordering::Relaxed)
    }

    pub(crate) fn checkpoint(
        &self,
        execution: ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
        units: u64,
    ) -> Result<(), CustomInvariantNativeCheckpointDenial> {
        if !self.admits_native_context(context) {
            return Err(CustomInvariantNativeCheckpointDenial::Source(
                StructuralWalkDenialKind::Read(StructuralReadError::OutsideDeclaredAccess),
            ));
        }
        interruption::check_live(execution)
            .map_err(StructuralWalkDenialKind::Interrupted)
            .map_err(CustomInvariantNativeCheckpointDenial::Source)?;
        if !self.try_debit_native_work(units) {
            return Err(CustomInvariantNativeCheckpointDenial::Source(
                self.refusal(StructuralReadError::WorkBudgetExceeded),
            ));
        }
        crate::execution::request_stage::checkpoint(context, units).map_err(|stop| {
            self.record_native_stop(stop);
            CustomInvariantNativeCheckpointDenial::Request(stop)
        })
    }

    pub(crate) fn refusal(&self, error: StructuralReadError) -> StructuralWalkDenialKind {
        use interruption::CustomInvariantInterruption;
        match self.leased_stop() {
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
            Some(CustomPreparationStop::Request(worth_execution::MapKernelStop::Cancelled)) => {
                StructuralWalkDenialKind::Interrupted(CustomInvariantInterruption::Cancelled)
            }
            Some(CustomPreparationStop::Request(
                worth_execution::MapKernelStop::DeadlineElapsed,
            )) => {
                StructuralWalkDenialKind::Interrupted(CustomInvariantInterruption::DeadlineElapsed)
            }
            _ => StructuralWalkDenialKind::Read(error),
        }
    }

    pub(super) fn stop(&self, reason: CustomPreparationStop) {
        if let Some(leased) = &self.leased {
            let mut stop = leased
                .first_stop
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            stop.get_or_insert(reason);
        }
        self.exhausted.store(true, Ordering::Relaxed);
    }

    pub(super) fn leased_stop(&self) -> Option<CustomPreparationStop> {
        self.leased.as_ref().and_then(|leased| {
            *leased
                .first_stop
                .lock()
                .unwrap_or_else(|error| error.into_inner())
        })
    }

    pub(super) fn safe_point(&self) -> bool {
        let Some(leased) = &self.leased else {
            return true;
        };
        if let PreparationControl::Stage(loan) = &leased.control {
            if let Err(cause) = loan.poll() {
                self.loan_failed(cause);
                return false;
            }
            return self.leased_stop().is_none();
        }
        if leased.control.is_cancelled() {
            self.stop(CustomPreparationStop::Cancelled);
            return false;
        }
        if leased.control.deadline_elapsed() {
            self.stop(CustomPreparationStop::DeadlineElapsed);
            return false;
        }
        self.leased_stop().is_none()
    }
    pub(super) fn try_debit_native_work(&self, units: u64) -> bool {
        if self.exceeded() || !self.safe_point() {
            return false;
        }
        if {
            let update_counter = &self.consumed;
            let mut current = update_counter.load(Ordering::Relaxed);
            loop {
                let Some(updated_value) = current
                    .checked_add(units)
                    .filter(|next| self.maximum.is_none_or(|maximum| *next <= maximum.get()))
                else {
                    break Err(current);
                };
                match update_counter.compare_exchange_weak(
                    current,
                    updated_value,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => current = reloaded_value,
                }
            }
        }
        .is_err()
        {
            self.stop(CustomPreparationStop::WorkExhausted);
            return false;
        }
        // The Native funding context already records these units in the active
        // request. Do not add them to LeasedCustomMeter.consumed_work again.
        true
    }
    pub(super) fn loan_failed(&self, cause: ExecutionWorkLoanDenial) {
        match cause {
            ExecutionWorkLoanDenial::Request(stop) => self.record_native_stop(stop),
            ExecutionWorkLoanDenial::ClosedScope | ExecutionWorkLoanDenial::ForeignScope => {
                self.stop(CustomPreparationStop::UncheckedCustomKernel)
            }
        }
    }
    fn record_native_stop(&self, stop: MapKernelStop) {
        if let Some(leased) = &self.leased {
            leased
                .request_stop
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .get_or_insert(stop);
        }
        self.stop(CustomPreparationStop::Request(stop));
    }
    fn admits_native_context(&self, context: &MapKernelContext<'_, '_>) -> bool {
        if !context.is_current() {
            return false;
        }
        match self.leased.as_ref().map(|meter| meter.control) {
            Some(PreparationControl::Stage(loan)) => loan.validate_context(context).is_ok(),
            _ => true,
        }
    }
}
