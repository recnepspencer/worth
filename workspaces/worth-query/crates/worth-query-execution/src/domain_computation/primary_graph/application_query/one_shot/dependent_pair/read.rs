//! Ordinary one-shot execution charged as a serial owner stage.

use super::*;
use crate::domain_computation::primary_graph::application_query::derived_view::reconstruction_denial as denial;
use crate::domain_computation::primary_graph::application_query::{
    read_execution::{read_execution_denial, WorthQueryApplicationReadExecutionDenialKind},
    WorthQueryApplicationBatchReadDenial, WorthQueryApplicationBatchResult,
};
use std::cell::{Cell, RefCell};
use worth_execution::ExecutionMemoryReservation;
use worth_execution::{ExecutionResourceLease, MapKernelContext};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryAuthorization;

pub(super) fn execute<S, Q, P, R, A, I, C>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    request: &ExecutionRequest<'_, '_>,
    plan: WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, A, I, C>,
    batch: &WorthQueryApplicationQueryBatchAdmission,
    root: EntityId,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<WorthQueryApplicationBatchResult<Q, R>, Denial>
where
    S: ApplicationSchema,
    R: WorthQueryApplicationProjection<S, Q>,
{
    if !matches!(
        plan.query.authorization(),
        WorthQueryInstalledApplicationQueryAuthorization::Public
    ) {
        return Err(Denial::AuthorizationRequired);
    }
    let validation = plan
        .query
        .validation_work_bound()
        .ok_or(Denial::WorkCounterOverflow)?;
    context
        .checkpoint(validation)
        .map_err(|stop| denial::kernel(stop, Some(root)))?;
    let bytes = plan
        .parameters
        .retained_owned_capacity_bytes()
        .and_then(|bytes| bytes.checked_add(std::mem::size_of_val(&plan)))
        .and_then(|bytes| {
            bytes.checked_add(
                plan.graph_read_plan()
                    .budget_check()
                    .max_inline_index_bytes(),
            )
        })
        .and_then(|bytes| {
            bytes.checked_add(
                plan.graph_read_plan()
                    .cost_estimate()
                    .supported()
                    .proof_bytes(),
            )
        })
        .and_then(|bytes| bytes.checked_add(plan.graph_authority_identity.capacity()))
        .and_then(|bytes| bytes.checked_add(plan.provider_identity.capacity()))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Denial::ChargedBytesOverflow)?;
    let _plan_hold =
        ExecutionMemoryReservation::reserve_in_scope(lease, bytes).map_err(denial::lease)?;
    let stopped = Cell::new(None);
    let meter = RefCell::new(context);
    let charge = |units: usize, subject: &str| {
        let result = u64::try_from(units)
            .map_err(|_| worth_execution::MapKernelStop::WorkCounterOverflow)
            .and_then(|units| meter.borrow_mut().checkpoint(units));
        result.map_err(|stop| {
            stopped.set(Some(stop));
            read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                subject,
            )
        })
    };
    let result = application.execute_application_query_one_shot_in_batch_metered(
        request,
        plan,
        batch,
        Some(&charge),
    );
    if let Some(stop) = stopped.get() {
        return Err(denial::kernel(stop, Some(root)));
    }
    result.map_err(|cause| match cause {
        WorthQueryApplicationBatchReadDenial::Resource(denial) => {
            Denial::BatchResource { root, denial }
        }
        WorthQueryApplicationBatchReadDenial::Execution(denial) => {
            Denial::ReadDenied { root, denial }
        }
    })
}
