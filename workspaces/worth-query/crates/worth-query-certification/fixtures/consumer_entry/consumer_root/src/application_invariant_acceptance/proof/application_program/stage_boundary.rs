//! Ordinary no-source custody survives request refusal and completes the original tree.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationProgramOutputSettlement;
use worth_query_host::facade::primary_graph::WorthQueryExecutionObservation;
use worth_query_topology_entry::{
    PlanarAlternateFinalToSummaryConnection, PlanarFinalToSummaryConnection,
    PlanarOutputToAlternateFinalConnection, PlanarOutputToFinalConnection,
};

mod no_source;

fn assert_complete_original_tree<Query>(
    settlement: &WorthQueryApplicationProgramOutputSettlement<Query>,
) {
    assert_eq!(settlement.root_producer_contacts_in_this_demand(), 1);
    assert_eq!(
        settlement.output_count(),
        4,
        "no output record is lost or duplicated on resume"
    );
    assert_eq!(
        settlement
            .outputs_for::<ConsumerSchema, PlanarOutputToFinalConnection>()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["anchor-a"]
    );
    assert_eq!(
        settlement
            .outputs_for::<ConsumerSchema, PlanarOutputToAlternateFinalConnection>()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["anchor-b"]
    );
    assert_eq!(
        settlement
            .outputs_for::<ConsumerSchema, PlanarFinalToSummaryConnection>()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["final:anchor-a"]
    );
    assert_eq!(
        settlement
            .outputs_for::<ConsumerSchema, PlanarAlternateFinalToSummaryConnection>()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["final:anchor-b"]
    );
}
