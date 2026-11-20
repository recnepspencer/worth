pub(crate) use worth_query_testkit_certification::facade::certification::program_example_authority::temporal_integrity_lease::*;

#[test]
fn temporal_integrity_lease_exact_budget_keeps_the_serial_verdict() {
    let serial = run(false, None, false);
    let paid = run(true, None, false);
    assert_eq!(committed(&paid), committed(&serial));
    let limit = *paid.work.iter().max().unwrap();
    let exact = run(true, Some(limit), false);
    assert_eq!(committed(&exact), committed(&serial));
    assert_eq!(exact.work, paid.work);
}

#[test]
fn temporal_integrity_lease_one_unit_short_is_the_work_stop() {
    let paid = run(true, None, false);
    committed(&paid);
    let short = run(
        true,
        Some(paid.work.iter().max().unwrap().checked_sub(1).unwrap()),
        false,
    );
    let Ok(Outcome::Commit(Uncommitted::ProductUnpublished(unpublished))) = &short.outcome else {
        panic!("{:?}", short.outcome)
    };
    assert_eq!(
        unpublished.cause(),
        worth_query_host::facade::primary_graph::ProductUnpublishedCause::OriginalObservationStopped(
            worth_query_host::facade::primary_graph::RelationalExecutionDenialCause::WorkExhausted
        )
    );
    assert_eq!(unpublished.owner_effect_count(), 1);
    assert!(unpublished
        .next_actions()
        .contains(&worth_query_host::facade::primary_graph::ProductUnpublishedNextAction::Inspect));
    assert!(unpublished.live_obligation_count() > 0);
    let observed = unpublished
        .inspect()
        .expect("the actual original owner remains accessible");
    assert_eq!(observed.cause(), unpublished.cause());
    assert_eq!(observed.owner_effect_count(), 1);
    assert_eq!(
        observed.live_obligation_count(),
        unpublished.live_obligation_count()
    );
}

#[test]
fn temporal_integrity_lease_cancelled_amendment_keeps_the_cancellation() {
    let stopped = run(true, None, true);
    let Err(denial) = stopped.outcome else {
        panic!("{:?}", stopped.outcome)
    };
    assert_eq!(
        denial.kind(),
        RequestKind::ExecutionRequest(Advancement::Interrupted(Interruption::Cancelled))
    );
}
