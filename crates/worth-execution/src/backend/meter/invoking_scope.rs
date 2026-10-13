//! Settle a scope carried on the invoking meter without clearing earlier questions.
use std::marker::PhantomData;

use super::{KernelContext, NestedStopOwner, ACTIVE_METER};

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
    if let Some(context) = &context {
        if let Some(owner) = context.settle_result(&result, |error| classify(context, error)) {
            context.meter.borrow_mut().nested_stop.record(owner);
        }
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
