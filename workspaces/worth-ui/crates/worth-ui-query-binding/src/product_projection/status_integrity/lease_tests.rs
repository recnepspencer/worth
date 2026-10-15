//! The UI's shipped status invariant runs at the typed source mutation door.
use super::super::{
    application_authentication::authenticate, application_runtime::install_status_application,
};
use crate::declaration::WorthUiStatusUpdateDenial;
use crate::{WorthUiScalarProjectionSourceRecord as Record, WorthUiStatusQueryRequest};
use std::{
    num::NonZeroUsize,
    time::{Duration, Instant},
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial as Advancement,
        WorthQueryManagedComputationInterruption as Interruption,
        WorthQueryManagedComputationResourceDenial as Resource,
    },
    application_entry::{
        WorthQueryApplicationMutationOutcome as Mutation, WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as RequestDenial,
        WorthQueryApplicationRequestMutationDenialKind as RequestKind,
        WorthQueryApplicationRetainedMutationOutcome as Retained,
    },
    primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        bound_advancement_requests_on_this_thread_for_test as bound,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryApplicationUncommitted as Uncommitted,
        WorthQueryExecutionPlacementForTest as Placement,
    },
};
struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
struct Case {
    outcome: Result<Retained<WorthUiStatusUpdateDenial, Record>, RequestDenial>,
    work: Vec<u64>,
}
fn run(leased: bool, ceiling: Option<u64>, cancel: bool) -> Case {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let _restore = Restore(place(Placement::Serial), bound(None));
    let application = install_status_application().unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(120)),
        cancellation.token(),
    );
    let principal = authenticate(application.installed_schema(), &scope).unwrap();
    let before = application
        .request(&principal, &scope)
        .query(WorthUiStatusQueryRequest::new("platform.pulse.status"))
        .execute()
        .unwrap();
    let [source] = before.observed_sources() else {
        panic!("the actual source record is unique")
    };
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
        cancellation.cancel();
    }
    let outcome = application
        .request(&principal, &scope)
        .advanced()
        .mutate(Record::new("ONLINE", 1).unwrap())
        .expect_source(source.clone())
        .idempotency(&1_u64)
        .execute_retained_in_program(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    let work = reports()
        .into_iter()
        .filter_map(|report| report.ok().map(|report| report.charged_work()))
        .collect();
    Case { outcome, work }
}
fn committed(case: &Case) -> &Record {
    match &case.outcome {
        Ok(Retained::Committed {
            receipt, result, ..
        }) => {
            let work = receipt
                .mutation_work()
                .expect("the shipped source retains its invariant execution");
            assert!(work.relational_invariant_execution_count() > 0);
            assert!(work.invariant_work_units() > 0);
            result
        }
        Ok(Retained::Other(other)) => panic!("{other:?}"),
        Err(denial) => panic!("{denial:?}"),
    }
}
#[test]
fn status_lease_exact_budget_keeps_the_serial_verdict() {
    let serial = run(false, None, false);
    let paid = run(true, None, false);
    assert_eq!(committed(&paid), committed(&serial));
    let exact_budget = *paid.work.iter().max().unwrap();
    let exact = run(true, Some(exact_budget), false);
    assert_eq!(committed(&exact), committed(&serial));
    assert_eq!(exact.work, paid.work);
}
#[test]
fn status_lease_one_unit_short_is_the_work_stop() {
    let paid = run(true, None, false);
    committed(&paid);
    let short = run(
        true,
        Some(paid.work.iter().max().unwrap().checked_sub(1).unwrap()),
        false,
    );
    match short.outcome {
 Ok(Retained::Other(Mutation::Commit(Uncommitted::Denied(denial))))=>
  assert!(matches!(denial.kind(),worth_query_host::facade::primary_graph::WorthQueryApplicationCommitDenialKind::ExecutionResource{denial:Resource::WorkExhausted,..})),
 Err(denial)=>panic!("{denial:?}"),
 Ok(Retained::Other(other))=>panic!("{other:?}"),
 Ok(Retained::Committed{..})=>panic!("one unit short must stop"),
 }
}
#[test]
fn status_lease_cancelled_source_keeps_the_cancellation() {
    let cancelled = run(true, None, true);
    let Err(denial) = cancelled.outcome else {
        panic!("a cancelled source must stop")
    };
    assert_eq!(
        denial.kind(),
        RequestKind::ExecutionRequest(Advancement::Interrupted(Interruption::Cancelled))
    );
}
