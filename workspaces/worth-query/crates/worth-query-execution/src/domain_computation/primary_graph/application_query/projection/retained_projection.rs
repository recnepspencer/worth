//! Opt-in custody of the original completed Native disclosed projection.
//! No body, DTO, authority, snapshot or comparison result is synthesized.
use std::sync::Arc;
use worth_execution::{ChargedBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_query_installation::facade::WorthQueryInstalledGraphReadContract;

use super::super::{
    disclosure::WorthQueryApplicationQueryGovernance,
    one_shot::{
        denial, outcome::source_storage, WorthQueryApplicationOneShotDenial,
        WorthQueryApplicationOneShotDenialKind,
    },
    read_execution::RawOneShotRows,
};
mod body;
mod comparison;
pub(in crate::domain_computation::primary_graph) use body::OriginalProjection;
pub(in crate::domain_computation::primary_graph) use comparison::{
    compare_original_projections, ProjectionComparisonDenial,
};
pub(in crate::domain_computation::primary_graph::application_query) mod storage;

pub(in crate::domain_computation::primary_graph::application_query) fn retain(
    execution: &ExecutionRequest<'_, '_>,
    raw: &mut RawOneShotRows,
    graph: &WorthQueryInstalledGraphReadContract,
    governance: &WorthQueryApplicationQueryGovernance,
    bound: bool,
    query: &str,
) -> Result<Arc<OriginalProjection>, WorthQueryApplicationOneShotDenial> {
    if !bound || graph.orbit().is_some() || graph.component_selection().is_some() {
        return Err(denial(
            WorthQueryApplicationOneShotDenialKind::ProjectionInspectionUnavailable,
            query,
            query,
        ));
    }
    if raw.has_more || raw.next_boundary.is_some() {
        return Err(denial(
            WorthQueryApplicationOneShotDenialKind::ProjectionInspectionIncomplete,
            query,
            query,
        ));
    }
    let mut decisions = ExecutionRequestArrayBuilder::allocate(
        *execution,
        graph
            .projections()
            .len()
            .saturating_add(graph.relations().len()),
    )
    .map_err(|cause| source_storage::denied(cause, query))?;
    for slot in graph
        .projections()
        .iter()
        .map(|field| field.slot_key_identity())
        .chain(
            graph
                .relations()
                .iter()
                .map(|relation| relation.slot_key_identity()),
        )
    {
        let disclosed = governance.is_disclosed(&slot);
        decisions
            .push((slot, disclosed))
            .map_err(|cause| source_storage::denied(cause, query))?;
    }
    let decisions = decisions
        .seal()
        .map_err(|cause| source_storage::denied(cause, query))?;
    let quote = raw
        .rows
        .additional_charged_bytes()
        .saturating_add(OriginalProjection::allocation_bytes());
    let memory = execution.reserve_retained_memory(quote).map_err(|cause| {
        source_storage::denied(
            worth_execution::ExecutionRequestArrayDenial::Request(cause),
            query,
        )
    })?;
    raw.rows
        .retain_projection(decisions, memory)
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationOneShotDenialKind::ForeignPlan,
                query,
                query,
            )
        })
}

#[cfg(test)]
mod tests;
