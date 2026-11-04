//! Admission lapse precedes the allocation and funding of an advancement.
use super::*;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[test]
fn cancelled_commit_is_refused_before_any_advancement_is_funded() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for cancelled in [false, true] {
        let world = installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
        let cancellation = WorthQueryCancellationSource::new();
        let request = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
        let program = world
            .application
            .with_host_advancement(|phase| {
                let principal =
                    authenticated_principal(&phase.test_host_request(), &world, &request);
                let account =
                    resolved_account(&phase.test_host_request(), &world, "open", &request);
                admitted_program(&phase, &world, &principal, &account, &request, "closed")
            })
            .unwrap();
        if cancelled {
            cancellation.cancel();
        }
        reports();
        let before = reads();
        let outcome = world.application.compare_and_commit_application(
            program,
            idempotency(182, 183),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
        let funded = reports();
        if cancelled {
            assert!(
                matches!(outcome, WorthQueryApplicationCommitOutcome::Cancelled),
                "{outcome:?}"
            );
            assert!(
                funded.is_empty(),
                "cancelled authority must never enter advancement funding: {funded:?}"
            );
            assert_eq!(reads(), before);
        } else {
            assert!(
                matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
                "{outcome:?}"
            );
            assert!(
                !funded.is_empty(),
                "the genuine fresh control enters funding"
            );
            assert!(reads() > before);
        }
    }
}

fn requested_program(
    world: &crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld,
    request: &WorthQueryRequestScope,
) -> program_fixture::Program {
    world
        .application
        .with_host_advancement(|phase| {
            let principal = authenticated_principal(&phase.test_host_request(), world, request);
            let account = resolved_account(&phase.test_host_request(), world, "open", request);
            admitted_program(&phase, world, &principal, &account, request, "closed")
        })
        .unwrap()
}

#[test]
fn entry_custody_refreshes_lapse_after_advancement_is_funded() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        None,
        cancellation.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let program = requested_program(&world, &request);
    let entry = world
        .application
        .validate_commit_program(program)
        .unwrap_or_else(|outcome| panic!("fresh: {outcome:?}"));
    reports();
    let before = reads();
    world
        .application
        .with_funded_application_advancement(
            &request,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            |phase| {
                cancellation.cancel();
                assert!(matches!(
                    entry.in_advancement(&phase),
                    Err(WorthQueryApplicationCommitOutcome::Cancelled)
                ));
                assert_eq!(
                    reads(),
                    before,
                    "a lapse while opening stops before provider work"
                );
            },
        )
        .unwrap();
    assert!(
        !reports().is_empty(),
        "the owner completed the advancement opened before cancellation"
    );
}
