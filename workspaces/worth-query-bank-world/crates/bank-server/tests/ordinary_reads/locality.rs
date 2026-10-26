use bank_server::{
    queries, BankApplicationOneShotDenialKind, BankApplicationQueryDenial, BankReadControls,
};

use super::fixture::{ordinary_read_world, over_budget_discovery_world, OWNER};
use crate::support::request_scope;

#[test]
fn discovery_and_account_reads_are_bounded_by_the_touched_neighborhood() {
    let baseline = ordinary_read_world("read-local-baseline", 0);
    let expanded = ordinary_read_world("read-local-expanded", 32);
    let baseline_owner = baseline.authenticate(OWNER);
    let expanded_owner = expanded.authenticate(OWNER);

    let baseline_accounts = baseline
        .world
        .runtime
        .query(queries::accounts())
        .as_principal(&baseline_owner)
        .controls(controls(8))
        .execute()
        .expect("baseline discovery should execute");
    let expanded_accounts = expanded
        .world
        .runtime
        .query(queries::accounts())
        .as_principal(&expanded_owner)
        .controls(controls(8))
        .execute()
        .expect("expanded discovery should execute");

    assert_eq!(baseline_accounts.rows(), expanded_accounts.rows());
    assert_eq!(
        baseline_accounts.receipt().inspect().ordinary_work_units(),
        expanded_accounts.receipt().inspect().ordinary_work_units()
    );
    assert_eq!(expanded_accounts.rows().len(), 2);
    let account_ids = expanded_accounts
        .rows()
        .iter()
        .map(|account| account.id())
        .collect::<Vec<_>>();
    let mut expected_ids = vec![expanded.personal_account, expanded.business_account];
    expected_ids.sort();
    assert_eq!(account_ids, expected_ids);

    let summary_result = expanded
        .world
        .runtime
        .query(queries::account_summary(expanded.personal_account))
        .as_principal(&expanded_owner)
        .controls(controls(1))
        .execute()
        .expect("installed account summary query should execute");
    let [summary] = summary_result.rows() else {
        panic!("account summary must return exactly one row");
    };
    assert_eq!(summary.current_balance().minor_units(), 7_500);
    assert_eq!(summary.available_balance().minor_units(), 7_500);
    assert!(summary_result
        .receipt()
        .inspect()
        .terminal_resources_released());
}

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

#[test]
fn account_discovery_root_union_work_grows_at_most_linearly() {
    let mut baseline = None;
    for count in [16, 64, 250] {
        let fixture = over_budget_discovery_world("read-root-union-scale", count);
        let actor = fixture.authenticate();
        let result = fixture
            .world
            .runtime
            .query(queries::accounts())
            .as_principal(&actor)
            .controls(controls(count))
            .execute()
            .expect("result-bounded discovery needs no caller work estimate");
        assert_eq!(result.rows().len(), count);
        let inspection = result.receipt().inspect();
        let work = inspection.ordinary_work_units();
        assert!(work > 0);
        assert!(inspection.terminal_resources_released());
        let (base_count, base_work) = *baseline.get_or_insert((count, work));
        assert!(
            work * base_count <= base_work * count * 4,
            "root-union work must remain linear with headroom: {count} rows cost {work}, baseline {base_count} cost {base_work}"
        );
    }
}

fn controls(maximum_results: usize) -> BankReadControls {
    BankReadControls::current(request_scope(), maximum_results).unwrap()
}
