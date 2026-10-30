pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::advancement_custody::*;

#[test]
fn wake_pump_refuses_before_clock_truth_reads_or_operation_reentry() {
    let host_request =
        worth_query_testkit_host::facade::host::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _observation = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = CourtroomWorld::publish("ready");
    let mut port = world.conditional_clock();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _placement = Restore(place(placement), bound(None));
        let before = world.clock_control.observations();
        let _ = port.observe(request_execution);
        assert!(
            world.clock_control.observations() > before,
            "admitted wake pump contacts its actual clock reader"
        );
        for zero_memory in [false, true] {
            let _restore = Restore(
                place(placement),
                bound(Some(worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    if zero_memory { Some(0) } else { None },
                    Some(if zero_memory { 8_000_000 } else { 0 }),
                ))),
            );
            let before = (
                reads(),
                world.clock_control.observations(),
                world.contacts.snapshot(),
            );
            reports();
            let primary_graph::WorthQueryConditionalClockObservationOutcome::Failed(failure) =
                port.observe(request_execution)
            else {
                panic!("the host policy refuses before the wake pump begins");
            };
            let primary_graph::WorthQueryConditionalClockObservationFailureKind::ExecutionRequest(
                cause,
            ) = failure.kind()
            else {
                panic!("the pump retains request admission");
            };
            if !zero_memory {
                assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
            } else {
                match placement {
                    Placement::Serial | Placement::Leased(_) => {
                        let Denial::Resource(Resource::MemoryLimit {
                            level,
                            requested,
                            admitted,
                        }) = cause
                        else {
                            panic!("the exact lease bytes are retained");
                        };
                        assert_eq!(level, worth_query_host::facade::application_contribution::WorthQueryMemoryLimitLevel::Policy);
                        assert!(requested > 0);
                        assert_eq!(admitted, 0);
                    }
                    Placement::World | Placement::Certified { .. } => {
                        unreachable!("this probe declares placement")
                    }
                }
            }
            assert_eq!(
                (
                    reads(),
                    world.clock_control.observations(),
                    world.contacts.snapshot()
                ),
                before
            );
            assert_eq!(reports(), vec![Err(cause)]);
        }
    }
}

pub(crate) mod conditional_delivery;
