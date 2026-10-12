use super::{primary_graph, world::CourtroomWorld};
use primary_graph::WorthQueryConditionalExecutionCause as Cause;
use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_host::facade::primary_graph::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Kind,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Read;

#[test]
fn conditional_operation_keeps_index_stops_at_every_consult() {
    super::test_read_stops::every_selected_consult_with_door(
        64_000_000,
        worth_query_execution::facade::primary_graph::conditional_intent_read_is_active_for_test,
        |counted| {
            let world = CourtroomWorld::publish("ready");
            let port = std::cell::RefCell::new(world.conditional_clock());
            world.clock_control.push(1, 5);
            counted(&|request| {
                let answer = port.borrow_mut().observe(request);
                match answer {
                    primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(
                        receipt,
                    ) => {
                        for row in receipt.execution_provenance() {
                            match row.cause() {
                            Some(Cause::Cancelled) => return Err(Read::Cancelled),
                            Some(Cause::TimedOut) => return Err(Read::DeadlineElapsed),
                            Some(Cause::ReadExecution(cause)) => return Err(cause),
                            Some(Cause::ApplicationExecutionDenied { cause: Err(Control::Cancelled), .. }) => return Err(Read::Cancelled),
                            Some(Cause::ApplicationExecutionDenied { cause: Err(Control::TimedOut), .. }) => return Err(Read::DeadlineElapsed),
                            Some(Cause::ApplicationExecutionDenied { cause: Ok(Kind::ExecutionResource { denial: Resource::WorkExhausted, .. }), .. }) => return Err(Read::WorkExhausted),
                            Some(Cause::BridgeConditional(worth_runtime_bridge::facade::BridgeConditionalDenialKind::ExecutionDenied(worth_runtime_bridge::facade::BridgeExecutionDenial::Cancelled))) => return Err(Read::Cancelled),
                            Some(Cause::BridgeConditional(worth_runtime_bridge::facade::BridgeConditionalDenialKind::ExecutionDenied(worth_runtime_bridge::facade::BridgeExecutionDenial::WorkCeiling))) => return Err(Read::WorkExhausted),
                            Some(Cause::Reentry(primary_graph::WorthQueryConditionalReentryFailure::ApplicationAttempt(primary_graph::WorthQueryApplicationAttemptDenialKind::Cancelled))) => return Err(Read::Cancelled),
                            Some(Cause::Reentry(primary_graph::WorthQueryConditionalReentryFailure::ApplicationAttempt(primary_graph::WorthQueryApplicationAttemptDenialKind::DeadlineExceeded))) => return Err(Read::DeadlineElapsed),
                            Some(Cause::Reentry(primary_graph::WorthQueryConditionalReentryFailure::ApplicationAttempt(primary_graph::WorthQueryApplicationAttemptDenialKind::ExecutionRequest(Kind::ExecutionResource { denial: Resource::WorkExhausted, .. })))) => return Err(Read::WorkExhausted),
                            Some(Cause::ApplicationCommitDenied(primary_graph::WorthQueryApplicationCommitDenialKind::ExecutionResource { denial: Resource::WorkExhausted, .. })) => return Err(Read::WorkExhausted),
                            Some(other) => panic!("unexpected operation cause: {other:?}"),
                            None => {},
                        }
                        }
                        assert_eq!(
                            receipt.failed_operation_count(),
                            0,
                            "healthy operation must not fail"
                        );
                        assert_eq!(
                            receipt.committed_operation_count(),
                            1,
                            "healthy due operation commits"
                        );
                        Ok(receipt.committed_operation_count())
                    }
                    primary_graph::WorthQueryConditionalClockObservationOutcome::Failed(
                        failure,
                    ) => panic!("observation failed: {:?}", failure.kind()),
                    _ => panic!("fresh due clock observation must be accepted"),
                }
            })
        },
        Read::Cancelled,
        Read::WorkExhausted,
    );
}
