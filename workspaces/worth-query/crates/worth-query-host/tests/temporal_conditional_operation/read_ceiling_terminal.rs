//! The shipped clock must preserve the DecisionReadSet work-stop terminal and cause.
use super::{primary_graph as graph, world::CourtroomWorld};
use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
fn observe(
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
        worth_execution::SerialMemoryBudget::new(64 * 1024 * 1024),
        source.token(),
        None,
    );
    let request = worth_execution::ExecutionRequest::serial(&host);
    let (answer, _) = request
        .run(worth_execution::ExecutionWorkCeiling::new(ceiling), |_| {
            port.observe(&request)
        })
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
#[test]
fn conditional_read_ceiling_answers_the_computation_terminal_by_value() {
    let (terminal, cause, charges) = observe(64_000_000);
    assert_eq!(
        terminal,
        graph::WorthQueryConditionalExecutionTerminal::Committed
    );
    assert_eq!(cause, None);
    let expected = (
        graph::WorthQueryConditionalExecutionTerminal::Failed,
        Some(
            graph::WorthQueryConditionalExecutionCause::ApplicationExecutionDenied {
                stage: graph::WorthQueryApplicationCommitDenialStage::DecisionReadSet,
                cause: Ok(
                    graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
                        denial: Resource::WorkExhausted,
                        partition_identity: None,
                        policy_ancestor: None,
                    },
                ),
            },
        ),
    );
    // Existing charged checkpoints select caller ceilings without fixing a
    // price or an ordinal. The expected stage, terminal and cause are fixed
    // independently: renaming the stop leaves no matching answer.
    let mut answered = false;
    for required in charges
        .into_iter()
        .filter(|n| *n > 0)
        .collect::<std::collections::BTreeSet<_>>()
    {
        let (terminal, cause, _) = observe(required - 1);
        if (terminal, cause) == expected {
            answered = true;
            break;
        }
    }
    assert!(
        answered,
        "the real DecisionReadSet ceiling answers {expected:?}"
    );
}
