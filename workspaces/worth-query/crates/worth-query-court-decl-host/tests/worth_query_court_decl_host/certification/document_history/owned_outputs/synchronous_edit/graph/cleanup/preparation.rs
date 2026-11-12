//! A real pre-append custody refusal must retain the first writer's failed abort.
use super::*;
use worth_relational::facade::{publication::AuthoredRetirementRefusal, transactions::RecordRef};
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = if worker {
        world::install_worker_only()
    } else {
        world::install()
    };
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(&host, &request, "a");
    second::settle(&host, &request, "a");
    let source = world::model(&host, host.current_world());
    let before = super::super::super::failure::checkpoint(&host);
    let original_head = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let schedule = host
        .runtime()
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let _corrupt = host
        .runtime()
        .corrupt_next_coupled_participant_for_test(true);
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"corrupt-writer-cleanup".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::CleanupRequired(failed),
    ) = outcome
    else {
        panic!("the first real writer's failed cleanup must remain owned: {outcome:?}")
    };
    let WorthQueryApplicationCommitOutcome::Denied(cause) = failed.outcome() else {
        panic!("original Native refusal: {:?}", failed.outcome())
    };
    let refusal = cause
        .authored_retirement_error()
        .expect("original publication custody refusal");
    assert_eq!(
        refusal.authored_retirement_refusal(),
        Some(AuthoredRetirementRefusal::ForeignCoupledOwner)
    );
    assert_eq!(
        refusal.context.affected_records,
        [RecordRef::Entity(source.native_id)]
    );
    worth_query_testkit_certification_operation::facade::certification::source_only::managed_cleanup::assert_native_failure(failed.cleanup().failure());
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1
    );
    assert_eq!(courts::native_head(&host), original_head);
    super::super::super::failure::assert_model_unchanged(&host, &before);
    drop(schedule);
    let (_, cleanup) = failed.into_parts();
    host.with_host_advancement(|phase| {
        cleanup
            .retry_in_advancement(&phase)
            .unwrap_or_else(|owner| panic!("lawful cleanup retry: {owner:?}"))
    })
    .unwrap();
    assert_eq!(host.runtime().active_application_attempts_for_test(), 0);
    assert_eq!(
        host.runtime().world_active_publication_attempts_for_test(),
        0
    );
    assert_eq!(courts::native_head(&host), original_head);
    super::super::super::failure::assert_model_unchanged(&host, &before);
}
#[test]
fn preparation_refusal_retains_the_actual_failed_writer_cleanup() {
    exercise(false)
}
#[test]
fn worker_only_preparation_refusal_retains_the_actual_failed_writer_cleanup() {
    exercise(true)
}
