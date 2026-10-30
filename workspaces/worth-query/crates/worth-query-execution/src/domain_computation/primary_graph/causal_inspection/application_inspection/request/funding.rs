//! One original advancement owns success, refusal and every nested temporary.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::WorthQueryAdvancementDenial;
pub(super) fn open<T, Schema: ApplicationSchema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    request: &WorthQueryRequestScope,
    allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    body: impl FnOnce(&ExecutionRequest<'_, '_>) -> Result<T, InspectionCause>,
) -> Result<T, InspectionCause> {
    let mut completed = None;
    let handoff =
        runtime.with_funded_application_advancement(request, allocation_policy, |phase| {
            completed = Some(
                match phase.checked_execution_request_for(&runtime.product_runtime) {
                    Ok(execution) => body(&execution),
                    Err(cause) => Err(InspectionCause::Advancement(cause)),
                },
            );
        });
    match (handoff, completed) {
        (Ok(()), Some(result)) => result,
        (Err(WorthQueryAdvancementDenial::NestedStopped), Some(Err(cause))) => Err(cause),
        (Err(cause), _) => Err(InspectionCause::Advancement(cause)),
        (Ok(()), None) => unreachable!("successful original advancement invokes inspection"),
    }
}
