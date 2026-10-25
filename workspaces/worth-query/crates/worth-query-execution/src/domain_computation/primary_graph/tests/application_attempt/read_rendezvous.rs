//! Cancellation raised inside a read reaches the public commit door by value.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    take_native_field_contacts, ReadRendezvous,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[test]
fn cancellation_inside_a_native_read_stops_the_public_commit_door() {
    let world = installed_authorization_world(true);
    let source = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() + Duration::from_secs(60)),
        source.token(),
    );
    let program = world
        .application
        .with_host_advancement(|phase| {
            let principal = authenticated_principal(&phase.test_host_request(), &world, &request);
            let account = resolved_account(&phase.test_host_request(), &world, "open", &request);
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "cancelled-at-read",
            )
        })
        .unwrap();
    let mut outcome = None;
    let opened = world
        .application
        .with_application_advancement(&request, |phase| {
            take_native_field_contacts();
            let _rendezvous = ReadRendezvous::cancel_next(source.clone());
            outcome = Some(
                world
                    .application
                    .compare_and_commit_application_in_advancement(
                        &phase,
                        program,
                        idempotency(95, 95),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    ),
            );
            assert_eq!(
                take_native_field_contacts(),
                1,
                "cancellation was raised inside the actual field read"
            );
        });
    assert_eq!(opened, Ok(()));
    assert!(matches!(
        outcome.unwrap(),
        WorthQueryApplicationCommitOutcome::Cancelled
    ));
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        0
    );
}
