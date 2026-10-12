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
        true,
        |at, answered_at, stop| {
            assert_eq!(
                answered_at, at,
                "a later consult must not rescue a folded stop"
            );
            assert_eq!(
                *stop,
                Stop::Operation(Terminal::ControlStopped, Cause::Cancelled)
            );
        },
        |required, stop| {
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
            let expected = [
                Stop::Operation(Terminal::Failed, Cause::BridgeConditional(worth_runtime_bridge::facade::BridgeConditionalDenialKind::ExecutionDenied(worth_runtime_bridge::facade::BridgeExecutionDenial::WorkCeiling))),
                Stop::Operation(Terminal::Failed, Cause::ApplicationCommitDenied(graph::WorthQueryApplicationCommitDenialKind::ExecutionResource { denial: Resource::WorkExhausted, partition_identity: None, policy_ancestor: None })),
                Stop::Operation(Terminal::Failed, Cause::ApplicationCommitDenied(graph::WorthQueryApplicationCommitDenialKind::ExecutionResource { denial: Resource::WorkExhausted, partition_identity: Some(1), policy_ancestor: None })),
                Stop::Operation(Terminal::Failed, Cause::ApplicationExecutionDenied { stage: Stage::DecisionReadSet, cause: Ok(kind) }),
                Stop::Operation(Terminal::Retryable, Cause::ApplicationCommitDenied(graph::WorthQueryApplicationCommitDenialKind::ExecutionResource { denial: Resource::WorkExhausted, partition_identity: None, policy_ancestor: None })),
                Stop::Operation(Terminal::Retryable, Cause::ApplicationCommitDenied(graph::WorthQueryApplicationCommitDenialKind::ExecutionNestedPatternStopped { partition_identity: None })),
            ];
            assert!(expected.contains(stop), "charge {required}: {stop:?}");
        },
    );
}
