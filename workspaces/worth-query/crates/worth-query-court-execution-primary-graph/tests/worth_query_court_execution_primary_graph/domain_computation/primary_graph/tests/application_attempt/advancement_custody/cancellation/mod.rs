//! Admission lapse precedes the allocation and funding of an advancement.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::court_doors_for_test::EntryCustodyDoorsOps;
use crate::domain_computation::primary_graph::tests::application_attempt::advancement_custody::*;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

#[test]
fn cancelled_commit_is_refused_before_any_advancement_is_funded() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    for cancelled in [false, true] {
        let world = installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
        let cancellation = WorthQueryCancellationSource::new();
        let request = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
        let program = world
            .application
            .with_host_advancement(|phase| {
                let principal =
                    authenticated_principal(&phase.test_host_request_for_test(), &world, &request);
                let account = resolved_account(
                    &phase.test_host_request_for_test(),
                    &world,
                    "open",
                    &request,
                );
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
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
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

pub fn requested_program(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    request: &WorthQueryRequestScope,
) -> worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::Program{
    world
        .application
        .with_host_advancement(|phase| {
            let principal =
                authenticated_principal(&phase.test_host_request_for_test(), world, request);
            let account =
                resolved_account(&phase.test_host_request_for_test(), world, "open", request);
            admitted_program(&phase, world, &principal, &account, request, "closed")
        })
        .unwrap()
}

#[test]
fn entry_custody_refreshes_lapse_after_advancement_is_funded() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
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
        .validate_commit_program_for_test(program)
        .unwrap_or_else(|outcome| panic!("fresh: {outcome:?}"));
    reports();
    let before = reads();
    world
        .application
        .with_funded_application_advancement(
            &request,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            |phase| {
                cancellation.cancel();
                assert!(matches!(
                    entry.in_advancement_for_test(&phase),
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
