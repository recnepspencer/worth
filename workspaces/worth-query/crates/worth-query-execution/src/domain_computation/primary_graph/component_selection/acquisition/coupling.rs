//! Actual coupling revision reads after the caller admits the whole cohort.
use super::*;
use crate::domain_computation::primary_graph::source_facts::{
    SourceFactReadDenial, SourceFactsBuilder,
};
use worth_relational::facade::{
    runtime::{RelationalAdjacencyDirection, RelationalPinnedRead},
    transactions::RelationalExecutionDenialCause,
};
pub(in crate::domain_computation::primary_graph) enum CouplingObservationDenial {
    Unavailable,
    Execution(RelationalExecutionDenialCause),
    Source(SourceFactReadDenial),
}
pub(in crate::domain_computation::primary_graph) fn observe_coupling(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    pin: &RelationalPinnedRead,
    layout: &ComponentSetLayout,
    aggregate: &AggregateObservation,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<SourceFacts, CouplingObservationDenial> {
    let projection = pin
        .project_snapshot(pin.snapshot())
        .ok_or(CouplingObservationDenial::Unavailable)?;
    let mut facts = SourceFactsBuilder::default();
    for member in aggregate.native().members() {
        AggregateObservation::checkpoint(context).map_err(CouplingObservationDenial::Execution)?;
        facts
            .observe_entity(execution, member)
            .map_err(CouplingObservationDenial::Execution)?;
        for (_, kind) in layout.sources.iter() {
            for direction in [
                RelationalAdjacencyDirection::Outgoing,
                RelationalAdjacencyDirection::Incoming,
            ] {
                AggregateObservation::checkpoint(context)
                    .map_err(CouplingObservationDenial::Execution)?;
                facts
                    .observe_adjacency(execution, &projection, member, *kind, direction)
                    .map_err(CouplingObservationDenial::Source)?;
            }
        }
    }
    facts
        .finish(execution)
        .map_err(CouplingObservationDenial::Execution)
}
