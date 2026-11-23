//! Dispatch codec work under its original Native request.
use super::codec_control::{final_denial, Denial, RelationalContentControl};
use worth_execution::ExecutionRequest;

pub(super) fn run<T>(
    execution: &ExecutionRequest<'_, '_>,
    operation: impl FnOnce(&mut RelationalContentControl<'_, '_, '_>) -> Result<T, Denial>,
) -> Result<T, Denial> {
    run_codec(execution, None, operation).0
}

fn run_codec<T>(
    execution: &ExecutionRequest<'_, '_>,
    mut source: Option<
        crate::validation::custom_rule::work_meter::native_rule_debit::NativeRuleDebit<'_>,
    >,
    operation: impl FnOnce(&mut RelationalContentControl<'_, '_, '_>) -> Result<T, Denial>,
) -> (
    Result<T, Denial>,
    Option<crate::validation::data::structural_walk_denial::StructuralWalkDenialKind>,
) {
    let mut result = None;
    let mut source_stop = None;
    let mut operation = Some(operation);
    let handoff = crate::execution::request_stage::optional_stage(
        execution,
        |context, _memory| {
            let mut control = RelationalContentControl::new(context);
            if let Some(debit) = source.take() {
                control.attach_source(*execution, debit);
            }
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
            source_stop = control.source_stop();
        },
        |_| 0,
    );
    let result = match (handoff, result) {
        (Ok(_), Some(result)) => result,
        (Ok(_), None) => Err(Denial::EncodingInconsistency),
        (Err(cause), result) => Err(final_denial(
            result.and_then(Result::err),
            Denial::Execution(cause),
        )),
    };
    (result, source_stop)
}

/// Keep the exact Source refusal outside codec vocabulary.
pub(super) fn run_source<T>(
    budget: &crate::validation::data::CustomInvariantReadBudget<'_>,
    operation: impl FnOnce(&mut RelationalContentControl<'_, '_, '_>) -> Result<T, Denial>,
) -> Result<T, crate::validation::candidate_source::ManagedIndexSourcePreparationDenial> {
    use crate::derived_relation::DerivedRelationReadDenial;
    use crate::validation::candidate_source::ManagedIndexSourcePreparationDenial as Read;
    let (result, source_stop) =
        run_codec(&budget.execution(), Some(budget.native_debit()), operation);
    if let Some(cause) = source_stop {
        return Err(DerivedRelationReadDenial::Structural(cause).into());
    }
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
    let budget = match (execution.is_leased(), budget) {
        (true, Some(budget)) => budget,
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
