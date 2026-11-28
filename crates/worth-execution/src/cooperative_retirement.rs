//! Request-scoped progress without waiting for a contested retirement owner.
use crate::{ExecutionRequest, ExecutionStop};
use std::sync::{Mutex, MutexGuard, TryLockError};

#[derive(Debug, PartialEq, Eq)]
pub enum CooperativeStep<R> {
    Continue,
    Busy,
    Stopped(ExecutionStop),
    Complete(R),
}

pub trait CooperativeRetireTask {
    type Receipt;

    fn advance(&mut self, request: &ExecutionRequest<'_, '_>) -> CooperativeStep<Self::Receipt>;
}

/// Retain the same active task across refusals and its original allocation
/// after completion. Owners with borrowed phase authority supply a step per call.
#[derive(Debug)]
pub enum CooperativeRetirementState<T, O, R = ()> {
    Retiring(Option<T>),
    Retired { receipt: R, original: O },
}

impl<T, O, R> CooperativeRetirementState<T, O, R> {
    pub fn advance<E>(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        finish: impl FnOnce(T) -> Result<O, (T, E)>,
    ) -> Result<CooperativeStep<()>, E>
    where
        T: CooperativeRetireTask<Receipt = R>,
    {
        self.advance_with(request, |task, request| Ok(task.advance(request)), finish)
    }

    pub fn advance_with<E>(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        step: impl FnOnce(&mut T, &ExecutionRequest<'_, '_>) -> Result<CooperativeStep<R>, E>,
        finish: impl FnOnce(T) -> Result<O, (T, E)>,
    ) -> Result<CooperativeStep<()>, E> {
        let Self::Retiring(slot) = self else {
            return Ok(CooperativeStep::Complete(()));
        };
        match step(
            slot.as_mut().expect("owner retains the active task"),
            request,
        )? {
            CooperativeStep::Continue => Ok(CooperativeStep::Continue),
            CooperativeStep::Busy => Ok(CooperativeStep::Busy),
            CooperativeStep::Stopped(stop) => Ok(CooperativeStep::Stopped(stop)),
            CooperativeStep::Complete(receipt) => {
                match finish(slot.take().expect("owner holds the completed task")) {
                    Ok(original) => {
                        *self = Self::Retired { receipt, original };
                        Ok(CooperativeStep::Complete(()))
                    }
                    Err((original, error)) => {
                        *slot = Some(original);
                        Err(error)
                    }
                }
            }
        }
    }
}

impl<R> CooperativeStep<R> {
    pub fn into_progress<E>(
        self,
        busy: E,
        stopped: impl FnOnce(ExecutionStop) -> E,
    ) -> Result<Option<R>, E> {
        match self {
            Self::Continue => Ok(None),
            Self::Busy => Err(busy),
            Self::Stopped(stop) => Err(stopped(stop)),
            Self::Complete(receipt) => Ok(Some(receipt)),
        }
    }
}

/// Drive successive nonwaiting steps. Each step uses the original request,
/// including when an owner has to resume a previously paid phase.
pub fn drain_cooperative_retirement<R, E>(
    request: &ExecutionRequest<'_, '_>,
    mut step: impl FnMut(&ExecutionRequest<'_, '_>) -> Result<CooperativeStep<R>, E>,
    busy: impl Fn() -> E,
    stopped: impl Fn(ExecutionStop) -> E,
) -> Result<R, E> {
    loop {
        match step(request)? {
            CooperativeStep::Continue => {}
            CooperativeStep::Busy => return Err(busy()),
            CooperativeStep::Stopped(stop) => return Err(stopped(stop)),
            CooperativeStep::Complete(receipt) => return Ok(receipt),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CooperativeRetirementLockDenial {
    Busy,
    Poisoned,
}

pub fn try_lock_retirement<T>(
    task: &Mutex<T>,
) -> Result<MutexGuard<'_, T>, CooperativeRetirementLockDenial> {
    task.try_lock().map_err(|error| match error {
        TryLockError::WouldBlock => CooperativeRetirementLockDenial::Busy,
        TryLockError::Poisoned(_) => CooperativeRetirementLockDenial::Poisoned,
    })
}

/// Acquire an owner's retirement cursor without waiting. These owners retain
/// the original cursor after poisoning, so its guard still permits preflight.
pub fn try_retirement_guard<G, E>(attempt: std::sync::TryLockResult<G>, busy: E) -> Result<G, E> {
    match attempt {
        Ok(guard) => Ok(guard),
        Err(TryLockError::Poisoned(error)) => Ok(error.into_inner()),
        Err(TryLockError::WouldBlock) => Err(busy),
    }
}
