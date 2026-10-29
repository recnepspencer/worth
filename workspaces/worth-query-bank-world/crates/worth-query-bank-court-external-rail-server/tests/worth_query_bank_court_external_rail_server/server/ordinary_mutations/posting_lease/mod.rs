pub(crate) use worth_query_bank_testkit_server::facade::ordinary_mutations::posting_lease::*;

#[test]
fn posting_lease_keeps_the_serial_verdict_and_work() {
    let serial = run(false, None, false);
    let paid = run(true, None, false);
    assert_eq!(committed(&paid), committed(&serial));
    assert_eq!(paid.work, serial.work);
}

#[test]
fn posting_lease_without_work_room_keeps_the_request_stop() {
    let stopped = run(true, Some(0), false);
    let Err(denial) = stopped.outcome else {
        panic!("{:?}", stopped.outcome)
    };
    assert!(matches!(
        denial,
        worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Execution(
            WorthQueryAdvancementDenial::Resource(Resource::WorkExhausted)
        ) | worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(
            WorthQueryApplicationRequestMutationDenial::ExecutionRequest(
                WorthQueryAdvancementDenial::Resource(Resource::WorkExhausted)
            )
        )
    ));
    assert_eq!(stopped.balance_after, stopped.balance_before);
}

#[test]
fn posting_lease_cancelled_transfer_keeps_the_cancellation() {
    let cancelled = run(true, None, true);
    let Err(denial) = cancelled.outcome else {
        panic!("{:?}", cancelled.outcome)
    };
    assert!(matches!(
        denial,
        worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Execution(
            WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
            )
        )
    ));
}
