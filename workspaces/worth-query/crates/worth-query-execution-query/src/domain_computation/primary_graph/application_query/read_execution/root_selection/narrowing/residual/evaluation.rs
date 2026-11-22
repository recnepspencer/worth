//! Foundation's resumable machine, sliced at the request's safe points.
//!
//! Each slice runs at most `MAX_SLICE_QUANTUM` semantic work units; its
//! actual charge is billed to the request meter before the next slice, and
//! every slice boundary checks cancellation and the deadline. Where slices
//! end never changes the verdict or the charged work.
use super::{projection_unavailable, RootSelectionWork, WHERE_FILTER_SUBJECT};
use crate::domain_computation::primary_graph::application_query::read_execution::{
    read_execution_denial, relational_read_denial, WorthQueryApplicationReadExecutionDenial,
    WorthQueryApplicationReadExecutionDenialKind,
};
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use worth_foundational::expression_api::{
    ExpressionDenialFamily, ExpressionInputs, ExpressionResource, ExpressionStep, MAX_SLICE_QUANTUM,
};
use worth_query_declaration::facade::application_query::ApplicationQueryWhereFilterProgram;
use worth_query_installation::facade::WorthQueryInstalledWhereFilter;

pub fn admits_row(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    filter: &WorthQueryInstalledWhereFilter,
    inputs: &ExpressionInputs,
    work: &mut RootSelectionWork<'_>,
) -> Result<bool, WorthQueryApplicationReadExecutionDenial> {
    let mut continuation = filter
        .program()
        .compiled()
        .start(inputs, &ApplicationQueryWhereFilterProgram::PROFILE)
        .map_err(|_| projection_unavailable())?;
    let mut billed = 0;
    loop {
        work.checkpoint(WHERE_FILTER_SUBJECT)?;
        match continuation.step(MAX_SLICE_QUANTUM) {
            ExpressionStep::Suspended(next) => {
                billed = bill(
                    execution,
                    work,
                    billed,
                    next.cost().used(ExpressionResource::SemanticWork),
                )?;
                continuation = next;
            }
            ExpressionStep::Complete(evaluation) => {
                bill(
                    execution,
                    work,
                    billed,
                    evaluation.cost().used(ExpressionResource::SemanticWork),
                )?;
                return match evaluation.into_result() {
                    Ok(value) => value.as_bool().ok_or_else(projection_unavailable),
                    Err(denial) if denial.family() == ExpressionDenialFamily::ResourceExceeded => {
                        Err(read_execution_denial(
                            WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                            WHERE_FILTER_SUBJECT,
                        ))
                    }
                    Err(_) => Err(projection_unavailable()),
                };
            }
        }
    }
}

/// Charges the slice's semantic work beyond what earlier slices billed, to
/// both the caller's execution lease and the read's own work account.
fn bill(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    work: &mut RootSelectionWork<'_>,
    billed: u64,
    used: u64,
) -> Result<u64, WorthQueryApplicationReadExecutionDenial> {
    let units = used.saturating_sub(billed);
    SourceFactsAllocation::work(execution, units)
        .map_err(|cause| relational_read_denial(cause, WHERE_FILTER_SUBJECT))?;
    work.charge_residual_filter(
        usize::try_from(units).unwrap_or(usize::MAX),
        WHERE_FILTER_SUBJECT,
    )?;
    Ok(used)
}
