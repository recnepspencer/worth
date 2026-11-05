//! Evidence that a commit takes, or a retry avoids, the managed application attempt path.
use super::*;

pub fn assert_unseen_commit_uses_the_managed_path(world: &AuthorizationWorld) {
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let phase_custody = &phase;

            authenticated_principal(
                &(phase_custody).test_host_request_for_test(),
                world,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let probe_account = world
        .application
        .with_host_advancement(|phase| {
            let phase_custody = &phase;

            resolved_account(
                &(phase_custody).test_host_request_for_test(),
                world,
                "unrelated",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let positive_control = world
        .application
        .with_host_advancement(|phase| {
            let phase_custody = &phase;

            admitted_program(
                phase_custody,
                world,
                &principal,
                &probe_account,
                &request,
                "positive-control",
            )
        })
        .expect("the fixture read owns its installed host call");
    let before_control = world.application.application_attempt_work_for_test();
    assert!(matches!(
        world.application.compare_and_commit_application(
            positive_control,
            idempotency(207, 208),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let control = world
        .application
        .application_attempt_work_for_test()
        .since_for_test(before_control);
    assert_eq!(
        (*control.retained_resolutions_for_test()), 3,
        "an unseen commit checks idempotency before registration, after registration, and under commit authority"
    );
    assert_eq!((*control.managed_bridge_plans_for_test()), 1);
    assert_eq!((*control.provider_session_readmissions_for_test()), 1);
    assert_eq!((*control.provider_session_preparations_for_test()), 1);
    assert_eq!((*control.attempt_registrations_for_test()), 1);
    assert_eq!((*control.overlay_stagings_for_test()), 1);
    assert_eq!((*control.invariant_state_loads_for_test()), 1);
    assert_eq!((*control.invariant_executions_for_test()), 1);
    assert_eq!((*control.staged_session_preparations_for_test()), 1);
    assert_eq!((*control.prepared_commits_for_test()), 1);
    assert_eq!((*control.attempt_aborts_for_test()), 0);
    assert_eq!((*control.managed_cleanups_for_test()), 1);
    assert_eq!((*control.external_dispatch_admissions_for_test()), 0);
    assert_eq!(
        world.application.provider_session_resource_count_for_test(),
        0
    );
}

pub fn assert_no_managed_attempt_work(
    work: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::provider::application_attempt_work::WorthQueryApplicationAttemptWorkSnapshot,
) {
    assert_eq!((*work.managed_bridge_plans_for_test()), 0);
    assert_eq!((*work.provider_session_readmissions_for_test()), 0);
    assert_eq!((*work.provider_session_preparations_for_test()), 0);
    assert_eq!((*work.staged_session_preparations_for_test()), 0);
    assert_eq!((*work.attempt_registrations_for_test()), 0);
    assert_eq!((*work.overlay_stagings_for_test()), 0);
    assert_eq!((*work.invariant_state_loads_for_test()), 0);
    assert_eq!((*work.invariant_executions_for_test()), 0);
    assert_eq!((*work.prepared_commits_for_test()), 0);
    assert_eq!((*work.attempt_aborts_for_test()), 0);
    assert_eq!((*work.managed_cleanups_for_test()), 0);
    assert_eq!((*work.external_dispatch_admissions_for_test()), 0);
}
