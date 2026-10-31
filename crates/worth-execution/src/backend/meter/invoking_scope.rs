//! Settle a scope carried on the invoking meter without clearing earlier questions.
use std::{marker::PhantomData, panic::AssertUnwindSafe};

use super::super::panic_boundary::contain;
use super::{KernelContext, KernelFailure, NestedStopOwner, ACTIVE_METER};

/// Prepare on the real invoking meter; no scope, report or work is recreated.
pub(crate) fn run_invoking_preparation<R, E>(
    operation: impl FnOnce() -> Result<R, E>,
) -> Result<Result<R, E>, crate::WorkCeilingDenial> {
    let Some(mut context) = ACTIVE_METER
        .with(|active| active.borrow().last().cloned())
        .map(|meter| KernelContext {
            meter,
            _lease: PhantomData,
        })
    else {
        return Err(crate::WorkCeilingDenial::Admission(
            crate::LeaseDenial::refused(crate::LeaseDenial::NoActiveExecutionScope),
        ));
    };
    let result = run_invoking_scope(
        || {
            let result = contain(AssertUnwindSafe(|| {
                context.poll()?;
                operation().map_err(KernelFailure::Domain)
            }))
            .unwrap_or(Err(KernelFailure::Panic));
            // The existing scope owner checks this after containing the
            // callback too: a panic cannot replace an already latched stop.
            if let Some(stop) = context.checkpoint_stop() {
                let _disposed = contain(AssertUnwindSafe(|| drop(result)));
                return Err(KernelFailure::Stop(stop));
            }
            match result {
                Ok(value) => match context.poll() {
                    Ok(()) => Ok(value),
                    Err(stop) => {
                        let _disposed = contain(AssertUnwindSafe(|| drop(value)));
                        Err(KernelFailure::Stop(stop))
                    }
                },
                Err(cause) => Err(cause),
            }
        },
        |context, failure| context.failure_owner(failure),
    );
    match result {
        Ok(value) => Ok(Ok(value)),
        Err(KernelFailure::Domain(cause)) => Ok(Err(cause)),
        Err(KernelFailure::Stop(stop)) => Err(crate::WorkCeilingDenial::Stopped(stop)),
        Err(KernelFailure::Panic) => Err(crate::WorkCeilingDenial::Panicked),
        Err(KernelFailure::ResultCapacityExceeded) => {
            unreachable!("native preparation has no payload capacity")
        }
    }
}

pub(crate) fn run_invoking_scope<R, E>(
    kernel: impl FnOnce() -> Result<R, E>,
    classify: impl FnOnce(&KernelContext<'_, '_>, &E) -> NestedStopOwner,
) -> Result<R, E> {
    let context = ACTIVE_METER
        .with(|active| active.borrow().last().cloned())
        .map(|meter| KernelContext {
            meter,
            _lease: PhantomData,
        });
    let questions = context.as_ref().map(InvokingQuestions::enter);
    let result = kernel();
    if let Some(context) = &context
        && let Some(owner) = context.settle_result(&result, |error| classify(context, error))
    {
        context.meter.borrow_mut().nested_stop.record(owner);
    }
    drop(questions);
    result
}

/// Pending questions before entry belong to the invoking kernel, not this scope.
/// On unwind, questions made in the scope remain pending for the panic boundary.
struct InvokingQuestions<'a> {
    context: &'a KernelContext<'a, 'a>,
    inherited: bool,
}

impl<'a> InvokingQuestions<'a> {
    fn enter(context: &'a KernelContext<'a, 'a>) -> Self {
        let inherited = std::mem::take(&mut context.meter.borrow_mut().pending_request_refusal);
        Self { context, inherited }
    }
}

impl Drop for InvokingQuestions<'_> {
    fn drop(&mut self) {
        self.context.meter.borrow_mut().pending_request_refusal |= self.inherited;
    }
}
