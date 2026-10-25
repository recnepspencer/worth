use bank_domain::model::CustomerRole;
use bank_server::{
    queries, BankApplicationOneShotDenialKind, BankApplicationQueryDenial, BankReadControls,
};

use super::fixture::{
    ordinary_read_world, ordinary_read_world_with_pending_payments,
    over_budget_discovery_world_with_role, APPROVER, OWNER, VIEWER,
};
use crate::support::request_scope;

#[test]
fn fixed_role_and_status_guards_are_query_owned_and_receipted() {
    let fixture = ordinary_read_world("pending-guard-authority", 0);
    let approver = fixture.authenticate(APPROVER);
    let result = fixture
        .world
        .runtime
        .query(queries::pending_payments())
        .as_principal(&approver)
        .controls(controls(8))
        .execute()
        .expect("an approver should discover approval-required payments");

    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].id(), fixture.payment);
    let inspection = result.receipt().inspect();
    assert_eq!(inspection.result_count(), 1);
    assert!(inspection.ordinary_work_units() > 0);
    assert!(inspection.terminal_resources_released());

    for principal in [OWNER, VIEWER] {
        let actor = fixture.authenticate(principal);
        let result = fixture
            .world
            .runtime
            .query(queries::pending_payments())
            .as_principal(&actor)
            .controls(controls(8))
            .execute()
            .expect("a lawful non-approver receives an empty result");
        assert!(result.rows().is_empty());
    }
}

#[test]
fn pending_roots_are_identity_ordered_and_result_limited_before_projection() {
    let fixture = ordinary_read_world_with_pending_payments("pending-order-limit", 0, 3);
    let approver = fixture.authenticate(APPROVER);
    let result = fixture
        .world
        .runtime
        .query(queries::pending_payments())
        .as_principal(&approver)
        .controls(controls(3))
        .execute()
        .expect("three pending payments should fit the exact result limit");
    let ids = result
        .rows()
        .iter()
        .map(|payment| payment.id())
        .collect::<Vec<_>>();
    assert!(ids.windows(2).all(|window| window[0] < window[1]));
    assert_eq!(ids.len(), fixture.payments.len());

    let denial = fixture
        .world
        .runtime
        .query(queries::pending_payments())
        .as_principal(&approver)
        .controls(controls(2))
        .execute();
    assert!(matches!(
        denial,
        Err(BankApplicationQueryDenial::Execution(denial))
            if denial.kind() == BankApplicationOneShotDenialKind::ResultLimitExceeded
    ));
}

#[test]
fn empty_guarded_frontier_work_grows_at_most_linearly() {
    let mut baseline = None;
    for count in [16, 64, 265] {
        let fixture = over_budget_discovery_world_with_role(
            "pending-guard-scale",
            count,
            CustomerRole::Approver,
        );
        let actor = fixture.authenticate();
        let result = fixture
            .world
            .runtime
            .query(queries::pending_payments())
            .as_principal(&actor)
            .controls(controls(1))
            .execute()
            .expect("guarded discovery needs no caller work estimate");
        assert!(result.rows().is_empty());
        let inspection = result.receipt().inspect();
        let work = inspection.ordinary_work_units();
        assert!(work > 0);
        assert!(inspection.terminal_resources_released());
        let (base_count, base_work) = *baseline.get_or_insert((count, work));
        assert!(
            work * base_count <= base_work * count * 4,
            "guarded-frontier work must remain linear with headroom: {count} accounts cost {work}, baseline {base_count} cost {base_work}"
        );
    }
}

fn controls(maximum_results: usize) -> BankReadControls {
    BankReadControls::current(request_scope(), maximum_results).unwrap()
}
