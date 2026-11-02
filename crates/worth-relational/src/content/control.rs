use super::RelationalContentDenial as Denial;
use crate::execution::request_stage::checkpoint;
use worth_execution::{ExecutionRequest, MapKernelContext};
#[cfg(test)]
mod tests;

// Only the SDK's generic nested-stop sentinel may yield to a classified inner
// refusal. A concrete original-request stop is authoritative at final handoff.
fn final_denial(inner: Option<Denial>, outer: Denial) -> Denial {
    match (inner, outer) {
        (
            Some(inner),
            Denial::Execution(crate::execution::RelationalExecutionDenialCause::NestedStopped),
        ) => inner,
        (_, outer) => outer,
    }
}

/// Borrowed original controls; codecs cannot replace this account or its first stop.
pub struct RelationalContentControl<'borrow, 'scope, 'lease> {
    context: &'borrow mut MapKernelContext<'scope, 'lease>,
    first_stop: Option<Denial>,
    source: Option<&'borrow crate::validation::data::CustomInvariantReadBudget<'borrow>>,
    source_stop: Option<crate::validation::data::StructuralWalkDenialKind>,
}

impl<'borrow, 'scope, 'lease> RelationalContentControl<'borrow, 'scope, 'lease> {
    fn new(context: &'borrow mut MapKernelContext<'scope, 'lease>) -> Self {
        Self {
            context,
            first_stop: None,
            source: None,
            source_stop: None,
        }
    }

    pub fn checkpoint(&mut self, units: u64) -> Result<(), Denial> {
        if let Some(cause) = &self.first_stop {
            return Err(*cause);
        }
        let result = if let Some(source) = self.source {
            use crate::validation::data::CustomInvariantNativeCheckpointDenial as Stop;
            match source.checkpoint_native(self.context, units) {
                Ok(()) => Ok(()),
                Err(Stop::Request(cause)) => Err(cause),
                Err(Stop::Source(cause)) => {
                    self.source_stop = Some(cause);
                    self.first_stop = Some(Denial::EncodingInconsistency);
                    return Err(Denial::EncodingInconsistency);
                }
            }
        } else {
            checkpoint(self.context, units)
        };
        match result {
            Ok(()) => Ok(()),
            Err(cause) => {
                let cause = Denial::Execution(cause.into());
                self.first_stop = Some(cause);
                Err(cause)
            }
        }
    }
}

pub(super) fn run<T>(
    execution: &ExecutionRequest<'_, '_>,
    operation: impl FnOnce(&mut RelationalContentControl<'_, '_, '_>) -> Result<T, Denial>,
) -> Result<T, Denial> {
    let mut result = None;
    let mut operation = Some(operation);
    let handoff = crate::execution::request_stage::optional_stage(
        execution,
        |context, _memory| {
            let mut control = RelationalContentControl::new(context);
            result = Some(match operation.take() {
                Some(operation) => control.checkpoint(0).and_then(|()| operation(&mut control)),
                None => Err(Denial::EncodingInconsistency),
            });
            // Preserve the original classified stop even if a codec swallowed it.
            if let Err(cause) = control.checkpoint(0) {
                let inner = result
                    .as_ref()
                    .and_then(|value| value.as_ref().err())
                    .copied();
                result = Some(Err(final_denial(inner, cause)));
            }
        },
        |_| 0,
    );
    match (handoff, result) {
        (Ok(_), Some(result)) => result,
        (Ok(_), None) => Err(Denial::EncodingInconsistency),
        (Err(cause), result) => Err(final_denial(
            result.and_then(Result::err),
            Denial::Execution(cause),
        )),
    }
}

/// Keep the exact Source refusal outside codec vocabulary; the sentinel never escapes.
pub(super) fn run_source<T>(
    budget: &crate::validation::data::CustomInvariantReadBudget<'_>,
    operation: impl FnOnce(&mut RelationalContentControl<'_, '_, '_>) -> Result<T, Denial>,
) -> Result<T, crate::validation::candidate_source::ManagedIndexSourcePreparationDenial> {
    use crate::derived_relation::DerivedRelationReadDenial;
    use crate::validation::candidate_source::ManagedIndexSourcePreparationDenial as Read;
    let mut result = None;
    let mut source_stop = None;
    let mut operation = Some(operation);
    let handoff = crate::execution::request_stage::optional_stage(
        &budget.execution(),
        |context, _memory| {
            let mut control = RelationalContentControl::new(context);
            control.source = Some(budget);
            result = Some(match operation.take() {
                Some(operation) => control.checkpoint(0).and_then(|()| operation(&mut control)),
                None => Err(Denial::EncodingInconsistency),
            });
            if let Err(cause) = control.checkpoint(0) {
                let inner = result
                    .as_ref()
                    .and_then(|value| value.as_ref().err())
                    .copied();
                result = Some(Err(final_denial(inner, cause)));
            }
            source_stop = control.source_stop;
        },
        |_| 0,
    );
    if let Some(cause) = source_stop {
        return Err(DerivedRelationReadDenial::Structural(cause).into());
    }
    let result = match (handoff, result) {
        (Ok(_), Some(result)) => result,
        (Ok(_), None) => Err(Denial::EncodingInconsistency),
        (Err(cause), result) => Err(final_denial(
            result.and_then(Result::err),
            Denial::Execution(cause),
        )),
    };
    result.map_err(|cause| match cause {
        Denial::Execution(cause) => Read::Source(DerivedRelationReadDenial::Request(cause)),
        cause => Read::Content(cause),
    })
}

/// Top-level content evidence settles once into the existing commit account.
/// Kernels remain unaware of the dispatcher and cannot recursively borrow it.
pub(crate) fn run_commit_stage<T>(
    execution: &ExecutionRequest<'_, '_>,
    budget: Option<&crate::execution::RequestWorkBudget>,
    operation: impl FnOnce() -> Result<T, Denial>,
) -> Result<T, Denial> {
    let budget = match (
        crate::authority::commit::pipeline::request_lease(execution),
        budget,
    ) {
        (Some(_), Some(budget)) => budget,
        _ => return operation(),
    };
    let mut result = None;
    let handoff = budget.run_stage(*execution, || result = Some(operation()));
    match (handoff, result) {
        (Ok(()), Some(result)) => result,
        (Ok(()), None) => Err(Denial::EncodingInconsistency),
        (Err(cause), result) => Err(final_denial(
            result.and_then(Result::err),
            Denial::Execution(cause.into()),
        )),
    }
}
