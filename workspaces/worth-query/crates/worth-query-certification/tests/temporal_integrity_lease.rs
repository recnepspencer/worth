//! The example's shipped invariant participates in a real leased amendment.
use super::product_workflow_support::application_entry::{
    AmendTemporalDenial, AmendTemporalResult,
};
use super::{amendment, principal, ExampleApplication};
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
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as RequestDenial,
        WorthQueryApplicationRequestMutationDenialKind as RequestKind,
    },
    primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        bound_advancement_requests_on_this_thread_for_test as bound,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryApplicationCommitDenialKind as CommitKind,
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
    outcome: Result<Outcome<AmendTemporalDenial, AmendTemporalResult>, RequestDenial>,
    work: Vec<u64>,
}
fn run(leased: bool, ceiling: Option<u64>, cancel: bool) -> Case {
    let _observation = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let _restore = Restore(place(Placement::Serial), bound(None));
    let mut app = ExampleApplication::publish("blocked");
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(120)),
        cancellation.token(),
    );
    let actor = principal(&app, &scope);
    reports();
    place(if leased {
        Placement::Leased(NonZeroUsize::MIN)
    } else {
        Placement::Serial
    });
    bound(ceiling.map(|units| {
        worth_foundational::ExecutionBudget::new(NonZeroUsize::MIN, None, Some(units))
    }));
    if cancel {
        cancellation.cancel();
    }
    let outcome = app
        .runtime
        .request(&actor, &scope)
        .advanced()
        .mutate(amendment("leased-amendment", 2))
        .without_source()
        .idempotency(&0x91766b_u64)
        .execute_in_program(
            &app.runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    let work = reports()
        .into_iter()
        .filter_map(|report| report.ok().map(|report| report.charged_work()))
        .collect();
    bound(None);
    place(Placement::Serial);
    app.runtime.close_conditional_runtime().unwrap();
    Case { outcome, work }
}
fn committed(case: &Case) -> u64 {
    let Ok(Outcome::Committed { receipt, result }) = &case.outcome else {
        panic!("{:?}", case.outcome)
    };
    assert_eq!(result.revision, 2);
    let work = receipt.mutation_work().unwrap();
    assert!(work.invariant_work_units() > 0);
    work.relational_invariant_execution_count() as u64
}
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
    let Ok(Outcome::Commit(Uncommitted::Denied(denial))) = short.outcome else {
        panic!("{:?}", short.outcome)
    };
    assert!(matches!(
        denial.kind(),
        CommitKind::ExecutionResource {
            denial: Resource::WorkExhausted,
            ..
        }
    ));
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
