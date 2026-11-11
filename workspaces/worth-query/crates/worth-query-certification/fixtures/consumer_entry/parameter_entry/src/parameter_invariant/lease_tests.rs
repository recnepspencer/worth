//! The shipped parameter invariant runs at its authorized application commit door.
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        bound_advancement_requests_on_this_thread_for_test as bound,
        place_managed_computations_on_this_thread_for_test as place,
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
fn run() -> Outcome {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let _restore = Restore(place(Placement::Serial), bound(None));
    let world = world::install();
    let source = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(120)),
        source.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let program = program::prepare(&world, &scope);
    reports();
    place(Placement::Leased(NonZeroUsize::MIN));
    let mut outcome = None;
    let _opening = world
        .application
        .with_application_advancement(&scope, |phase| {
            source.cancel();
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
    outcome.expect("the actual parameter commit door ran")
}
#[test]
fn parameter_lease_cancelled_commit_keeps_the_cancellation() {
    let stopped = run();
    let Outcome::Cancelled = stopped else {
        panic!("{stopped:?}");
    };
}
