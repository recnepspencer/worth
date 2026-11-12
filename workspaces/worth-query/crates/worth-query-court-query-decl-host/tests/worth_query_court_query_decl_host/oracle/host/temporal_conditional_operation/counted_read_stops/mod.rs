pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::counted_read_stops::*;

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
        |_, stop| assert_work_stop_preserves_its_owning_phase(stop),
    );
}
