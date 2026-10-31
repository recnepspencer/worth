use super::*;
use crate::basis::WorthQueryProductObservationLease;
use worth_runtime_world::facade::ProductBranchCurrentnessFailure;

/// Validate through the same Native commit lane and World head guard as adoption.
/// Prepared values enter and leave the guard without allocation or authored effects.
pub(super) fn confirm<S: ApplicationSchema, Value>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    phase: &WorthQueryAdvancementPhase<'_>,
    product: &WorthQueryProductObservationLease,
    value: Value,
) -> Result<Value, (Value, Denial)> {
    let execution = match phase.execution_request_for(&runtime.product_runtime) {
        Ok(execution) => execution,
        Err(_) => return Err((value, Denial::ForeignPhase)),
    };
    if let Err(stop) = execution.check_live() {
        return Err((value, stopped(stop)));
    }
    let lane = runtime
        .primary_provider
        .application_branch_commit_lane_for_occurrence(
            product.observation().lifecycle_incarnation(),
        );
    let _coordination = lane.enter();
    runtime
        .primary_provider
        .graph
        .with_runtime(&execution, |native| {
            if let Err(denial) = native
                .owner_component_services()
                .basis_port()
                .compare_current_exact(product.relational_basis())
            {
                return Err((value, Denial::NativeBasis(denial)));
            }
            let world = runtime.product_runtime.owner.observation_port();
            match world.while_product_branch_current(
                product.observation(),
                value,
                |value, _current| match execution.check_live() {
                    Ok(()) => Ok(value),
                    Err(stop) => Err((value, stopped(stop))),
                },
            ) {
                Ok(outcome) => outcome,
                Err(ProductBranchCurrentnessFailure::ExpectedHeadUnavailable(value)) => {
                    Err((value, Denial::HeadDiverged))
                }
                Err(ProductBranchCurrentnessFailure::AdmissionDenied { denial, argument }) => {
                    Err((argument, Denial::HeadAdmission(denial)))
                }
            }
        })
}

fn stopped(stop: worth_execution::MapKernelStop) -> Denial {
    Denial::Stopped(
        crate::domain_computation::primary_graph::application_contribution::advancement_denial(
            worth_execution::WorkCeilingDenial::Stopped(stop),
        ),
    )
}
