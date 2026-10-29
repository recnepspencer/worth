//! The shipped clock must preserve the DecisionReadSet work-stop terminal and cause.
pub use super::{primary_graph as graph, world::CourtroomWorld};
pub use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
pub fn observe(
    ceiling: u64,
) -> (
    graph::WorthQueryConditionalExecutionTerminal,
    Option<graph::WorthQueryConditionalExecutionCause>,
    Vec<u64>,
) {
    let world = CourtroomWorld::publish("ready");
    let mut port = world.conditional_clock();
    world.clock_control.push(1, 5);
    let _query = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let observer = super::test_read_stops::observe_consults_for_test(|_| {});
    let source = worth_execution::CancellationSource::new();
    let host = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        source.token(),
        None,
    );
    let request = worth_execution::ExecutionRequest::serial(&host);
    let (answer, _) = request
        .run(
            worth_execution::ExecutionWorkCeiling::new(Some(ceiling)),
            |_| port.observe(&request),
        )
        .unwrap();
    let graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) = answer else {
        panic!("clock answer")
    };
    let row = receipt.execution_provenance().first().unwrap();
    let charges = super::test_read_stops::charged_prefixes_for_test(&observer)
        .into_iter()
        .map(|(_, required)| required)
        .collect();
    (row.terminal(), row.cause(), charges)
}
