//! A live SDK scope lends charging to its genuine current descendants.
use super::{KernelContext, KernelStop, ACTIVE_METER};
use crate::{ExecutionMemoryReservation, ExecutionRequest, LeaseDenial, WorkCeilingDenial};
use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub(super) struct WorkLoanAffinity {
    parent: Option<Arc<WorkLoanAffinity>>,
    live: AtomicBool,
    _memory: Option<ExecutionMemoryReservation>,
}
impl WorkLoanAffinity {
    pub(super) fn close(&self) {
        self.live.store(false, Ordering::Release);
    }
}

/// Charges only an actual active descendant of the issuing live SDK scope.
/// This owns no independent counter, ceiling, workers or allocation authority.
#[derive(Clone)]
pub struct ExecutionWorkLoan(Arc<WorkLoanAffinity>);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionWorkLoanDenial {
    ClosedScope,
    ForeignScope,
    Request(KernelStop),
}
impl KernelContext<'_, '_> {
    /// Whether this is the actual current meter, rather than a retained ancestor.
    pub fn is_current(&self) -> bool {
        self.is_active()
    }

    /// Issue once from this actual active scope; pay its cell before allocation.
    pub fn work_loan(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionWorkLoan, WorkCeilingDenial> {
        if !self.is_active() {
            return Err(WorkCeilingDenial::Admission(
                LeaseDenial::NoActiveExecutionScope,
            ));
        }
        if let Some(affinity) = &self.meter.borrow().issued_work_loan {
            return if affinity.live.load(Ordering::Acquire) {
                Ok(ExecutionWorkLoan(affinity.clone()))
            } else {
                Err(WorkCeilingDenial::Admission(
                    LeaseDenial::NoActiveExecutionScope,
                ))
            };
        }
        let mut parent = self.meter.borrow().limits.work_loan_affinity.clone();
        while let Some(affinity) = parent {
            if !affinity.live.load(Ordering::Acquire) {
                return Err(WorkCeilingDenial::Admission(
                    LeaseDenial::NoActiveExecutionScope,
                ));
            }
            parent = affinity.parent.clone();
        }
        let bytes = size_of::<(usize, usize, WorkLoanAffinity)>() as u64;
        let memory = execution.reserve_retained_memory(bytes)?;
        let mut meter = self.meter.borrow_mut();
        let affinity = Arc::new(WorkLoanAffinity {
            parent: meter.limits.work_loan_affinity.clone(),
            live: AtomicBool::new(true),
            _memory: memory,
        });
        meter.limits.work_loan_affinity = Some(affinity.clone());
        meter.issued_work_loan = Some(affinity.clone());
        Ok(ExecutionWorkLoan(affinity))
    }
}
impl ExecutionWorkLoan {
    pub fn validate_context(
        &self,
        context: &KernelContext<'_, '_>,
    ) -> Result<(), ExecutionWorkLoanDenial> {
        if !self.0.live.load(Ordering::Acquire) {
            return Err(ExecutionWorkLoanDenial::ClosedScope);
        }
        if !context.is_active() {
            return Err(ExecutionWorkLoanDenial::ForeignScope);
        }
        let mut ancestor = context.meter.borrow().limits.work_loan_affinity.clone();
        let mut found = false;
        let mut live = true;
        while let Some(affinity) = ancestor {
            found |= Arc::ptr_eq(&affinity, &self.0);
            live &= affinity.live.load(Ordering::Acquire);
            ancestor = affinity.parent.clone();
        }
        if !found {
            return Err(ExecutionWorkLoanDenial::ForeignScope);
        }
        if !live {
            return Err(ExecutionWorkLoanDenial::ClosedScope);
        }
        Ok(())
    }
    pub fn checkpoint_in(
        &self,
        context: &mut KernelContext<'_, '_>,
        units: u64,
    ) -> Result<(), ExecutionWorkLoanDenial> {
        self.validate_context(context)?;
        let result = if context.has_accounting() {
            context.checkpoint(units)
        } else {
            context.poll()
        };
        result.map_err(ExecutionWorkLoanDenial::Request)
    }
    pub fn checkpoint(&self, units: u64) -> Result<(), ExecutionWorkLoanDenial> {
        self.with_current(|context| self.checkpoint_in(context, units))
    }
    pub fn poll(&self) -> Result<(), ExecutionWorkLoanDenial> {
        self.with_current(|context| {
            self.validate_context(context)?;
            context.poll().map_err(ExecutionWorkLoanDenial::Request)
        })
    }
    fn with_current<T>(
        &self,
        use_context: impl FnOnce(&mut KernelContext<'_, '_>) -> Result<T, ExecutionWorkLoanDenial>,
    ) -> Result<T, ExecutionWorkLoanDenial> {
        if !self.0.live.load(Ordering::Acquire) {
            return Err(ExecutionWorkLoanDenial::ClosedScope);
        }
        let meter = ACTIVE_METER
            .with(|active| active.borrow().last().cloned())
            .ok_or(ExecutionWorkLoanDenial::ForeignScope)?;
        use_context(&mut KernelContext {
            meter,
            _lease: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests;
