//! The shipped posting rule is exercised through Bank's transfer entry.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::primary_graph::{
    advancement_requests_on_this_thread_for_test as reports,
    bound_advancement_requests_on_this_thread_for_test as bound,
    place_managed_computations_on_this_thread_for_test as place,
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
    outcome: Result<
        worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt,
        worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial,
    >,
    work: Vec<u64>,
    balance_before: i64,
    balance_after: i64,
}
fn run(leased: bool, ceiling: Option<u64>, cancel: bool) -> Case {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let _restore = Restore(place(Placement::Serial), bound(None));
    let fixture = ordinary_read_world("posting-lease", 0);
    let owner = fixture.authenticate(OWNER);
    let source = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(120)),
        source.token(),
    );
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
    let outcome = fixture
        .world
        .runtime
        .mutate(mutations::send_money(send_for(&fixture)))
        .as_principal(&owner)
        .controls(BankMutationControls::new(
            scope.clone(),
            key("posting-lease"),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("transfer construction: {cause:?}"));
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
    let request = fixture.world.runtime.request(&owner, &scope);
    let outcome = outcome.advance(&request, NonZeroUsize::new(20_000).unwrap());
    let work = reports()
        .into_iter()
        .filter_map(|report| report.ok().map(|report| report.charged_work()))
        .collect();
    bound(None);
    place(Placement::Serial);
    let close_scope = request_scope();
    let close = fixture.world.runtime.request(&owner, &close_scope);
    let outcome = match outcome {
        worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            completed,
        ) => {
            let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
                receipt,
                ..
            } = completed.evidence()
            else {
                panic!("fresh posting evidence")
            };
            let receipt = receipt.clone();
            assert!(completed.close(&close).is_ok());
            Ok(receipt)
        }
        worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused {
            operation,
            denial,
        } => {
            let _runtime = operation.into_runtime_for_cleanup();
            Err(denial)
        }
        _ => panic!("unexpected native posting custody"),
    };
    let balance_after = balance();
    Case {
        outcome,
        work,
        balance_before,
        balance_after,
    }
}
fn committed(case: &Case) -> u64 {
    let Ok(receipt) = &case.outcome else {
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

use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
