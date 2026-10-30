//! Source visits charge their original SDK scope immediately, including refused paths.
use crate::derived_relation::DerivedRelationReadDenial;
use crate::validation::candidate_source::ManagedIndexSourcePreparationDenial as Denial;
use crate::validation::custom_rule::CustomPreparationBudget;
use worth_execution::{ExecutionRequest, MapKernelStop, WorkCeilingDenial};
fn request(cause: WorkCeilingDenial) -> Denial {
    DerivedRelationReadDenial::Request(cause.into()).into()
}
pub(super) fn prepare(
    execution: ExecutionRequest<'_, '_>,
    prepare: impl FnOnce(&CustomPreparationBudget) -> Result<(), Denial>,
) -> Result<(), Denial> {
    let mut answer = None;
    let handoff = execution.run_native_with_checkpoint(|context| {
        answer = Some(
            match CustomPreparationBudget::source_stage(&execution, context) {
                Ok(budget) => {
                    let result = prepare(&budget);
                    match budget.request_stop() {
                        Some(stop) => Err(request(WorkCeilingDenial::Stopped(stop))),
                        None => result,
                    }
                }
                Err(cause) => Err(request(cause)),
            },
        );
    });
    match (handoff, answer) {
        (Ok(_), Some(answer)) => answer,
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause)))
            if cause.execution_cause().is_some() =>
        {
            Err(cause)
        }
        (Err(cause), _) => Err(request(cause)),
        (Ok(_), None) => unreachable!("completed original Source stage"),
    }
}
