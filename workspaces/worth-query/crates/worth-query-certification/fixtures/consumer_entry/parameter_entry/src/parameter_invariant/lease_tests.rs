//! The shipped parameter invariant runs at its authorized application commit door.
use std::{
    num::NonZeroUsize,
    time::{Duration, Instant},
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial as Advancement,
        WorthQueryManagedComputationResourceDenial as Resource,
    },
    primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        bound_advancement_requests_on_this_thread_for_test as bound,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryApplicationCommitDenialKind as Denial,
        WorthQueryApplicationCommitOutcome as Outcome, WorthQueryApplicationIdempotencyBinding,
        WorthQueryExecutionPlacementForTest as Placement,
    },
};
#[path = "lease_tests/authentication.rs"]
mod authentication;
#[path = "lease_tests/program.rs"]
mod program;
#[path = "lease_tests/resources.rs"]
mod resources;
#[path = "lease_tests/schema.rs"]
mod schema;
#[path = "lease_tests/world.rs"]
mod world;
struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
struct Case {
    outcome: Outcome,
    opening: Result<(), Advancement>,
    work: Option<u64>,
}
fn run(leased: bool, ceiling: Option<u64>, cancel: bool) -> Case {
    let _restore = Restore(place(Placement::Serial), bound(None));
    let world = world::install();
    let source = WorthQueryCancellationSource::new();
    let scope =
        WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(120), source.token());
    let program = program::prepare(&world, &scope);
    reports();
    place(if leased {
        Placement::Leased(NonZeroUsize::MIN)
    } else {
        Placement::Serial
    });
    bound(ceiling.map(|units| {
        worth_foundational::ExecutionBudget::new(NonZeroUsize::MIN, 64 * 1024 * 1024, units)
    }));
    let mut outcome = None;
    let opening = world
        .application
        .with_application_advancement(&scope, |phase| {
            if cancel {
                source.cancel();
            }
            outcome = Some(
                world
                    .application
                    .compare_and_commit_application_in_advancement(
                        &phase,
                        program,
                        WorthQueryApplicationIdempotencyBinding::for_host_commit::<
                            schema::LeaseSchema,
                            schema::SetCount,
                            _,
                            _,
                        >(&198_u64, &8_u64)
                        .unwrap(),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    ),
            );
        });
    let requests = reports();
    assert_eq!(requests.len(), 1);
    Case {
        outcome: outcome.expect("the actual parameter commit door ran"),
        opening,
        work: requests[0]
            .as_ref()
            .ok()
            .map(|report| report.charged_work()),
    }
}
fn committed(case: &Case) {
    assert!(
        matches!(case.outcome, Outcome::Committed(_)),
        "{:?}",
        case.outcome
    );
    assert_eq!(case.opening, Ok(()));
}
#[test]
fn parameter_lease_exact_budget_keeps_the_serial_verdict() {
    committed(&run(false, None, false));
    let paid = run(true, None, false);
    committed(&paid);
    let exact = run(true, paid.work, false);
    committed(&exact);
    assert_eq!(exact.work, paid.work);
}
#[test]
fn parameter_lease_one_unit_short_is_the_work_stop() {
    let paid = run(true, None, false);
    committed(&paid);
    let short = run(
        true,
        Some(paid.work.unwrap().checked_sub(1).unwrap()),
        false,
    );
    let Outcome::Denied(denial) = short.outcome else {
        panic!("{:?}", short.outcome)
    };
    assert!(matches!(
        denial.kind(),
        Denial::ExecutionResource {
            denial: Resource::WorkExhausted,
            ..
        }
    ));
}
#[test]
fn parameter_lease_cancelled_commit_keeps_the_cancellation() {
    let stopped = run(true, None, true);
    let Outcome::Cancelled = stopped.outcome else {
        panic!("{:?}", stopped.outcome);
    };
}
