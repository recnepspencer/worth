//! Exact Native and World current-head confirmation, shared by read owners.
use super::CurrentReadDenial as Denial;
use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::ProductBranchCurrentnessFailure;

/// Internal callers carry the execution already checked by their advancement.
pub(in crate::domain_computation::primary_graph::product_operation) fn confirm_in_execution<
    S: ApplicationSchema,
    Value,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    product: &WorthQueryProductObservationLease,
    request: Option<
        &crate::domain_computation::authorization::admitted_request::AdmittedRequestCurrentness,
    >,
    value: Value,
) -> Result<
    (
        Value,
        Option<crate::domain_computation::authorization::WorthQueryAdmissionLapse>,
    ),
    (Value, Denial),
> {
    confirm_in_execution_with_effect(runtime, execution, product, request, value, |_| {})
}

/// Only the document owner supplies this platform metadata transition.
pub(in crate::domain_computation::primary_graph::product_operation) fn confirm_in_execution_with_effect<
    S: ApplicationSchema,
    Value,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    product: &WorthQueryProductObservationLease,
    request: Option<
        &crate::domain_computation::authorization::admitted_request::AdmittedRequestCurrentness,
    >,
    value: Value,
    effect: impl FnOnce(&mut Value),
) -> Result<
    (
        Value,
        Option<crate::domain_computation::authorization::WorthQueryAdmissionLapse>,
    ),
    (Value, Denial),
> {
    if let Err(stop) = execution.check_live() {
        return Err((value, stopped(stop)));
    }
    let lane = runtime
        .runtime
        .primary_provider
        .application_branch_commit_lane_for_occurrence(
            product.observation().lifecycle_incarnation(),
        );
    let _coordination = lane.enter();
    runtime
        .runtime
        .primary_provider
        .graph
        .with_runtime(execution, |native| {
            confirm_in_native_with_effect(
                runtime,
                execution,
                product,
                request,
                &_coordination,
                native,
                value,
                effect,
            )
        })
}

/// The caller holds this occurrence's lane, and supplies the actual Native owner.
/// This core never acquires Query's Native transport or an earlier-order seal.
pub(in crate::domain_computation::primary_graph::product_operation) fn confirm_in_native<
    S: ApplicationSchema,
    Value,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    product: &WorthQueryProductObservationLease,
    request: Option<
        &crate::domain_computation::authorization::admitted_request::AdmittedRequestCurrentness,
    >,
    coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
    native: &worth_relational::facade::runtime::RelationalRuntime,
    value: Value,
) -> Result<
    (
        Value,
        Option<crate::domain_computation::authorization::WorthQueryAdmissionLapse>,
    ),
    (Value, Denial),
> {
    confirm_in_native_with_effect(
        runtime,
        execution,
        product,
        request,
        coordination,
        native,
        value,
        |_| {},
    )
}

fn confirm_in_native_with_effect<S: ApplicationSchema, Value>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    product: &WorthQueryProductObservationLease,
    request: Option<
        &crate::domain_computation::authorization::admitted_request::AdmittedRequestCurrentness,
    >,
    coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
    native: &worth_relational::facade::runtime::RelationalRuntime,
    value: Value,
    effect: impl FnOnce(&mut Value),
) -> Result<
    (
        Value,
        Option<crate::domain_computation::authorization::WorthQueryAdmissionLapse>,
    ),
    (Value, Denial),
> {
    if !coordination.admits(product.observation()) {
        return Err((value, Denial::HeadDiverged));
    }
    if let Err(denial) = native
        .owner_component_services()
        .basis_port()
        .compare_current_exact(product.relational_basis())
    {
        return Err((value, Denial::NativeBasis(denial)));
    }
    let world = runtime.runtime.product_runtime.owner.observation_port();
    match world.while_product_branch_current(product.observation(), value, |mut value, _current| {
        if let Err(stop) = execution.check_live() {
            return Err((value, stopped(stop)));
        }
        // Denial allocation and refused-value cleanup belong outside owner guards.
        let lapse = request.and_then(|request| request.lapse());
        if lapse.is_none() {
            effect(&mut value);
        }
        Ok((value, lapse))
    }) {
        Ok(outcome) => outcome,
        Err(ProductBranchCurrentnessFailure::ExpectedHeadUnavailable(value)) => {
            Err((value, Denial::HeadDiverged))
        }
        Err(ProductBranchCurrentnessFailure::AdmissionDenied { denial, argument }) => {
            Err((argument, Denial::HeadAdmission(denial)))
        }
    }
}

fn stopped(stop: worth_execution::MapKernelStop) -> Denial {
    Denial::Stopped(
        crate::domain_computation::primary_graph::application_contribution::advancement_denial(
            worth_execution::WorkCeilingDenial::Stopped(stop),
        ),
    )
}
