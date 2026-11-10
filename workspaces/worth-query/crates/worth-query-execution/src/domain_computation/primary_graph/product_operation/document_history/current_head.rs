//! History projects the shared current-head owner's typed failures.
use super::WorthQueryDocumentHistoryDenial as Denial;
use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::authorization::{
    admitted_request::AdmittedRequestCurrentness, WorthQueryAdmissionLapse,
};
use crate::domain_computation::primary_graph::product_operation::current_read::{
    head, CurrentReadDenial,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;

pub(in crate::domain_computation::primary_graph) fn confirm<S: ApplicationSchema, T>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    phase: &WorthQueryAdvancementPhase<'_>,
    product: &WorthQueryProductObservationLease,
    value: T,
) -> Result<T, (T, Denial)> {
    confirm_with_request(runtime, phase, product, None, value).map(|(value, _)| value)
}
pub(super) fn confirm_with_request<S: ApplicationSchema, T>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    phase: &WorthQueryAdvancementPhase<'_>,
    product: &WorthQueryProductObservationLease,
    request: Option<&AdmittedRequestCurrentness>,
    value: T,
) -> Result<(T, Option<WorthQueryAdmissionLapse>), (T, Denial)> {
    let execution = match phase.execution_request_for(&runtime.runtime.product_runtime) {
        Ok(execution) => execution,
        Err(_) => return Err((value, Denial::ForeignPhase)),
    };
    head::confirm_in_execution(runtime, &execution, product, request, value)
        .map_err(|(value, cause)| (value, document_denial(cause)))
}
pub(super) fn confirm_in_execution_with_effect<S: ApplicationSchema, T>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    product: &WorthQueryProductObservationLease,
    request: Option<&AdmittedRequestCurrentness>,
    value: T,
    effect: impl FnOnce(&mut T),
) -> Result<(T, Option<WorthQueryAdmissionLapse>), (T, Denial)> {
    head::confirm_in_execution_with_effect(runtime, execution, product, request, value, effect)
        .map_err(|(value, cause)| (value, document_denial(cause)))
}
pub(super) fn document_denial(cause: CurrentReadDenial) -> Denial {
    match cause {
        CurrentReadDenial::HeadDiverged => Denial::HeadDiverged,
        CurrentReadDenial::NativeBasis(cause) => Denial::NativeBasis(cause),
        CurrentReadDenial::HeadAdmission(cause) => Denial::HeadAdmission(cause),
        CurrentReadDenial::Stopped(cause) => Denial::Stopped(cause),
        CurrentReadDenial::Authorization(_) | CurrentReadDenial::Busy => {
            unreachable!("head owner returns request lapses as data and never tries a lane")
        }
    }
}
