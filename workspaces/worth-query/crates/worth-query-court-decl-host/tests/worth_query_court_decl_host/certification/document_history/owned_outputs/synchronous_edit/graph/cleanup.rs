//! Failed disposal of an abandoned real writer retains its native owner.
use super::*;
mod preparation;
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_value::<unstable::program::Program>(limits, 2);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::failure::checkpoint(&host);
    let original_head = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let schedule = host
        .runtime()
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 3,
        })
        .without_source()
        .idempotency(&"abandoned-graph-cleanup".to_owned())
        .execute_performed::<unstable::program::Program, unstable::program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::CleanupRequired { cleanup, outcome } =
        outcome
    else {
        panic!("an abandoned writer's failed native cleanup must remain owned")
    };
    let native = match *outcome {
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(native) => native,
        WorthQueryApplicationPerformedMutationOutcome::Blocked(blocked) => panic!(
            "inner source is blocked: {:?}; contacts={}; cleanup={:?}; run={}",
            blocked.outcome(),
            host.producer_contacts_on_this_thread_for_test() - contacts,
            cleanup.failure().kind(),
            cleanup.failure().run_identity()
        ),
        WorthQueryApplicationPerformedMutationOutcome::CleanupRequired { .. } => {
            panic!("multiple nested cleanup owners")
        }
        _ => panic!("an unpublished refusal cannot perform or retain output publication"),
    };
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        cause,
    )) = native
    else {
        panic!("cleanup must preserve the original whole-edit refusal: {native:?}")
    };
    let failure = cause
        .invariant_execution_failure()
        .expect("original discovery-support refusal");
    assert_eq!(failure.kind(),worth_query_host::facade::primary_graph::WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged);
    assert!(
        failure
            .detail()
            .contains("owned-outputs.unstable-discovery-binding.v1"),
        "{failure:?}"
    );
    worth_query_testkit_certification_operation::facade::certification::source_only::managed_cleanup::assert_native_failure(cleanup.failure());
    assert!(!cleanup.failure().run_identity().is_empty());
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
    assert_eq!(courts::native_head(&host), original_head);
    super::super::failure::assert_model_unchanged(&host, &before);
    drop(schedule);
    host.with_host_advancement(|phase| {
        cleanup
            .retry_in_advancement(&phase)
            .unwrap_or_else(|owner| panic!("lawful cleanup retry: {owner:?}"))
    })
    .unwrap();
    assert_eq!(
        courts::native_head(&host),
        original_head,
        "disposal cannot commit the refused edit"
    );
    super::super::failure::assert_model_unchanged(&host, &before);
    assert_eq!(
        host.runtime().active_application_attempts_for_test(),
        0,
        "all actual writer sessions are discharged"
    );
    assert_eq!(
        host.runtime().world_active_publication_attempts_for_test(),
        0
    );
}
#[test]
fn abandoned_graph_writer_keeps_failed_native_cleanup_until_lawful_retry() {
    exercise(false)
}
#[test]
fn worker_only_abandoned_graph_writer_keeps_failed_native_cleanup_until_lawful_retry() {
    exercise(true)
}
