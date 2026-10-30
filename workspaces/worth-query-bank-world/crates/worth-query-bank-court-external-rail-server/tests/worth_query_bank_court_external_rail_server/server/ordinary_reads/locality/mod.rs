pub(crate) use worth_query_bank_testkit_server::facade::ordinary_reads::locality::*;

#[test]
fn activity_limit_is_enforced_and_reported_by_the_public_result() {
    let fixture = ordinary_read_world("read-activity-limit", 0);
    let owner = fixture.authenticate(OWNER);
    let activity = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .page(BankReadControls::current(request_scope(), 1).unwrap())
        .expect("account activity page should execute");

    assert_eq!(activity.rows()[0].entries().len(), 1);
    assert!(activity.continuation().is_some());
    assert!(activity.receipt().inspect().terminal_resources_released());
}

#[test]
fn account_discovery_result_limit_denies_before_projecting_an_oversized_union() {
    let fixture = over_budget_discovery_world("read-work-budget", 160);
    let actor = fixture.authenticate();
    let outcome = fixture
        .world
        .runtime
        .query(queries::accounts())
        .as_principal(&actor)
        .controls(controls(159))
        .execute();

    assert!(matches!(
        outcome,
        Err(BankApplicationQueryDenial::Execution(denial))
            if denial.kind() == BankApplicationOneShotDenialKind::ResultLimitExceeded
    ));
}
