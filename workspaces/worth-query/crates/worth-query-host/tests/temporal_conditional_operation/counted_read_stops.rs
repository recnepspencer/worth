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
            assert_eq!(emitted_at, at, "the first emission is the injected consult");
            assert_eq!(
                *stop,
                Stop::Operation(Terminal::ControlStopped, Cause::Cancelled)
            );
        },
        |required, stop| assert_eq!(*stop, expected_work(516), "last charge {required}"),
    );
}

fn expected_work(ordinal: usize) -> Stop {
    use graph::{
        WorthQueryApplicationCommitDenialStage as Stage,
        WorthQueryProviderSessionDenialKind as Kind,
    };
    use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
    let kind = Kind::ExecutionResource {
        denial: Resource::WorkExhausted,
        partition_identity: None,
        policy_ancestor: None,
    };
    // This one-intent workload has 203 Bridge charges, 232 authority charges,
    // and 81 provider charges. On a stopped run the four native point charges
    // are ledger positions 227..=230; provider nested admission is position 483.
    // Use the SDK ledger position, because source identities change encoded prices.
    let expected = match ordinal {
        1..=203 => Stop::Operation(
            Terminal::Failed,
            Cause::BridgeConditional(
                worth_runtime_bridge::facade::BridgeConditionalDenialKind::ExecutionDenied(
                    worth_runtime_bridge::facade::BridgeExecutionDenial::WorkCeiling,
                ),
            ),
        ),
        227..=230 => Stop::Operation(
            Terminal::Failed,
            Cause::ApplicationExecutionDenied {
                stage: Stage::DecisionReadSet,
                cause: Ok(Kind::ExecutionResource {
                    denial: Resource::WorkExhausted,
                    partition_identity: Some(1),
                    policy_ancestor: None,
                }),
            },
        ),
        204..=435 => Stop::Operation(
            Terminal::Failed,
            Cause::ApplicationExecutionDenied {
                stage: Stage::DecisionReadSet,
                cause: Ok(kind),
            },
        ),
        483 => Stop::Operation(
            Terminal::Retryable,
            Cause::ApplicationCommitDenied(
                graph::WorthQueryApplicationCommitDenialKind::ExecutionNestedPatternStopped {
                    partition_identity: None,
                },
            ),
        ),
        436..=516 => Stop::Operation(
            Terminal::Retryable,
            Cause::ApplicationCommitDenied(
                graph::WorthQueryApplicationCommitDenialKind::ExecutionResource {
                    denial: Resource::WorkExhausted,
                    partition_identity: None,
                    policy_ancestor: None,
                },
            ),
        ),
        unknown => panic!("the fixture has no expected charge {unknown}"),
    };

    expected
}

#[test]
fn conditional_operation_work_stops_at_every_charge_by_value() {
    fn run(ceiling: u64) -> (Terminal, Option<Cause>, Vec<u64>) {
        let _query = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
        worth_query_execution::facade::primary_graph::place_managed_computations_on_this_thread_for_test(worth_query_execution::facade::primary_graph::WorthQueryExecutionPlacementForTest::Serial);
        let world = CourtroomWorld::publish("ready");
        let mut port = world.conditional_clock();
        world.clock_control.push(1, 5);
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
    assert_eq!(charges.len(), 516, "fixture charge ledger");
    for required in charges
        .into_iter()
        .filter(|n| *n > 0)
        .collect::<std::collections::BTreeSet<_>>()
    {
        let (terminal, cause, attempted) = run(required - 1);
        assert_eq!(
            Stop::Operation(terminal, cause.expect("a denied charge has its cause")),
            expected_work(attempted.len()),
            "charge {required}"
        );
    }
}
