//! The shipped posting rule is exercised through Bank's transfer entry.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::primary_graph::{
    advancement_requests_on_this_thread_for_test as reports,
    bound_advancement_requests_on_this_thread_for_test as bound,
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryApplicationUncommitted as Uncommitted,
    WorthQueryExecutionPlacementForTest as Placement,
};
struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
struct Case {
    outcome: bank_server::BankMoneyMovementExecution,
    work: Vec<u64>,
    balance_before: i64,
    balance_after: i64,
}
fn run(leased: bool, ceiling: Option<u64>, cancel: bool) -> Case {
    let _restore = Restore(place(Placement::Serial), bound(None));
    let fixture = ordinary_read_world("posting-lease", 0);
    let owner = fixture.authenticate(OWNER);
    let source = WorthQueryCancellationSource::new();
    let scope =
        WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(120), source.token());
    let balance = || {
        fixture
            .world
            .runtime
            .query(queries::account_summary(fixture.personal_account))
            .as_principal(&owner)
            .controls(read_controls())
            .execute()
            .unwrap()
            .into_rows()[0]
            .current_balance()
            .minor_units()
    };
    let balance_before = balance();
    reports();
    place(if leased {
        Placement::Leased(NonZeroUsize::MIN)
    } else {
        Placement::Serial
    });
    bound(ceiling.map(|work| {
        worth_foundational::ExecutionBudget::new(NonZeroUsize::MIN, 64 * 1024 * 1024, work)
    }));
    if cancel {
        source.cancel();
    }
    let outcome = fixture
        .world
        .runtime
        .mutate(mutations::send_money(send_for(&fixture)))
        .as_principal(&owner)
        .controls(BankMutationControls::new(scope, key("posting-lease")))
        .execute();
    let work = reports()
        .into_iter()
        .filter_map(|report| report.ok().map(|report| report.charged_work()))
        .collect();
    bound(None);
    place(Placement::Serial);
    let balance_after = balance();
    Case {
        outcome,
        work,
        balance_before,
        balance_after,
    }
}
fn committed(case: &Case) -> u64 {
    let Ok(WorthQueryApplicationMutationOutcome::Committed { receipt, result: _ }) = &case.outcome
    else {
        panic!("{:?}", case.outcome)
    };
    let work = receipt
        .mutation_work()
        .expect("the actual transfer retains its invariant execution");
    assert!(work.relational_invariant_execution_count() > 0);
    assert!(work.invariant_work_units() > 0);
    const TRANSFER_MINOR_UNITS: i64 = 25;
    assert_eq!(
        case.balance_after,
        case.balance_before - TRANSFER_MINOR_UNITS
    );
    work.relational_invariant_execution_count() as u64
}
#[test]
fn posting_lease_exact_budget_keeps_the_serial_verdict() {
    let serial = run(false, None, false);
    let paid = run(true, None, false);
    assert_eq!(committed(&paid), committed(&serial));
    let exact_budget = *paid
        .work
        .iter()
        .max()
        .expect("the transfer opens bounded requests");
    let exact = run(true, Some(exact_budget), false);
    assert_eq!(committed(&exact), committed(&serial));
    assert_eq!(exact.work, paid.work);
}
#[test]
fn posting_lease_one_unit_short_is_the_work_stop() {
    let paid = run(true, None, false);
    committed(&paid);
    let short = run(
        true,
        Some(paid.work.iter().max().unwrap().checked_sub(1).unwrap()),
        false,
    );
    let Ok(WorthQueryApplicationMutationOutcome::Commit(Uncommitted::Denied(denial))) =
        short.outcome
    else {
        panic!("{:?}", short.outcome)
    };
    assert!(matches!(denial.kind(),worth_query_host::facade::primary_graph::WorthQueryApplicationCommitDenialKind::ExecutionResource{denial:Resource::WorkExhausted,..}));
}
#[test]
fn posting_lease_cancelled_transfer_keeps_the_cancellation() {
    let cancelled = run(true, None, true);
    let Err(denial) = cancelled.outcome else {
        panic!("{:?}", cancelled.outcome)
    };
    assert_eq!(denial.kind(),worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind::ExecutionRequest(
 WorthQueryAdvancementDenial::Interrupted(WorthQueryManagedComputationInterruption::Cancelled)));
}

use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
