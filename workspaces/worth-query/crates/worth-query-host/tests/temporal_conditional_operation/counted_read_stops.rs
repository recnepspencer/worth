use super::{primary_graph as graph, world::CourtroomWorld};
use graph::{
    WorthQueryConditionalExecutionCause as Cause,
    WorthQueryConditionalExecutionTerminal as Terminal,
};
#[derive(Debug, Eq, PartialEq)]
enum Stop {
    Operation(Terminal, Cause),
    Clock(graph::WorthQueryConditionalClockObservationFailureKind),
}
#[test]
fn conditional_operation_keeps_index_stops_at_every_consult() {
    let _query = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    worth_query_execution::facade::primary_graph::place_managed_computations_on_this_thread_for_test(worth_query_execution::facade::primary_graph::WorthQueryExecutionPlacementForTest::Serial);
    super::test_read_stops::selected_consult_answers(
        64_000_000,
        worth_query_execution::facade::primary_graph::conditional_intent_read_is_active_for_test,
        |counted| {
            let world = CourtroomWorld::publish("ready");
            let port = std::cell::RefCell::new(world.conditional_clock());
            world.clock_control.push(1, 5);
            counted(&|request| match port.borrow_mut().observe(request) {
                graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) => {
                    for row in receipt.execution_provenance() {
                        if let Some(cause) = row.cause() {
                            return Err(Stop::Operation(row.terminal(), cause));
                        }
                        assert_eq!(row.terminal(), Terminal::Committed);
                    }
                    assert_eq!(receipt.failed_operation_count(), 0);
                    assert_eq!(receipt.committed_operation_count(), 1);
                    Ok(receipt.committed_operation_count())
                }
                graph::WorthQueryConditionalClockObservationOutcome::Failed(failure) => {
                    Err(Stop::Clock(failure.kind()))
                }
                graph::WorthQueryConditionalClockObservationOutcome::Duplicate(_)
                | graph::WorthQueryConditionalClockObservationOutcome::Stale
                | graph::WorthQueryConditionalClockObservationOutcome::Reordered
                | graph::WorthQueryConditionalClockObservationOutcome::Closed => {
                    panic!("fresh due clock observation")
                }
            })
        },
        false,
        |at, emitted_at, stop| {
            assert!(emitted_at >= at, "the emission follows the injection");
            assert_eq!(
                *stop,
                Stop::Operation(Terminal::ControlStopped, Cause::Cancelled),
                "injected at {at}, answered at {emitted_at}"
            );
        },
        |_, stop| assert_eq!(*stop, provider_work_stop()),
    );
}

fn provider_work_stop() -> Stop {
    use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
    Stop::Operation(
        Terminal::Retryable,
        Cause::ApplicationCommitDenied(
            graph::WorthQueryApplicationCommitDenialKind::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: None,
                policy_ancestor: None,
            },
        ),
    )
}

#[test]
fn conditional_operation_work_stop_terminal_matches_its_typed_cause() {
    fn run(ceiling: u64) -> (Terminal, Option<Cause>, Vec<u64>) {
        let _query = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
        worth_query_execution::facade::primary_graph::place_managed_computations_on_this_thread_for_test(worth_query_execution::facade::primary_graph::WorthQueryExecutionPlacementForTest::Serial);
        let world = CourtroomWorld::publish("ready");
        let mut port = world.conditional_clock();
        world.clock_control.push(1, 5);
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
        (
            row.terminal(),
            row.cause(),
            super::test_read_stops::charged_prefixes_for_test(&observer)
                .into_iter()
                .map(|(_, n)| n)
                .collect(),
        )
    }
    let (terminal, cause, charges) = run(64_000_000);
    assert_eq!((terminal, cause), (Terminal::Committed, None));
    assert!(!charges.is_empty(), "the real door charges work");
    for required in charges
        .into_iter()
        .filter(|n| *n > 0)
        .collect::<std::collections::BTreeSet<_>>()
    {
        let (terminal, cause, attempted) = run(required - 1);
        // No fitted charge ordinals: pin the terminal contract for each
        // typed cause, and use the dedicated read-ceiling proofs for exact
        // stage and cause values. This cannot prove per-charge attribution.
        let expected_terminal = match cause.expect("a denied charge has its cause") {
            Cause::BridgeConditional(worth_runtime_bridge::facade::BridgeConditionalDenialKind::ExecutionDenied(
                worth_runtime_bridge::facade::BridgeExecutionDenial::WorkCeiling,
            )) | Cause::ApplicationExecutionDenied {
                stage: graph::WorthQueryApplicationCommitDenialStage::DecisionReadSet
                    | graph::WorthQueryApplicationCommitDenialStage::InvariantExecution,
                cause: Ok(graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
                    denial: worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted,
                    partition_identity: None | Some(0) | Some(1), policy_ancestor: None,
                }),
            } => Terminal::Failed,
            Cause::ApplicationCommitDenied(
                graph::WorthQueryApplicationCommitDenialKind::ExecutionResource {
                    denial: worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted,
                    partition_identity: None | Some(0) | Some(1), policy_ancestor: None,
                } | graph::WorthQueryApplicationCommitDenialKind::ExecutionNestedPatternStopped {
                    partition_identity: None | Some(0) | Some(1),
                },
            ) => Terminal::Retryable,
            other => panic!("unexpected work-stop cause at a real charge: {other:?}"),
        };
        assert_eq!(terminal, expected_terminal, "charge {required}");
        assert!(!attempted.is_empty());
    }
}
