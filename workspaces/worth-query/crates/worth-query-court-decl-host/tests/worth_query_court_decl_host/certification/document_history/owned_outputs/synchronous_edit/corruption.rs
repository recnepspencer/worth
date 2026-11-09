//! Corruption changes a real prepared writer; the ordinary Native fence refuses it.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::{publication::AuthoredRetirementRefusal, transactions::RecordRef};

fn refuses_foreign_writer(host: &world::Host) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(host, &request, "a");
    second::settle(host, &request, "a");
    let source = world::model(host, host.current_world());
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let before = failure::checkpoint(host);
    let _fault = host
        .runtime()
        .corrupt_next_coupled_participant_for_test(true);
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"corrupt-synchronous-edit".to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("a foreign publication must refuse the entire edit: {outcome:?}")
    };
    let error = denial
        .authored_retirement_error()
        .expect("the Native publication refusal");
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(AuthoredRetirementRefusal::ForeignCoupledOwner)
    );
    assert_eq!(
        error.context.affected_records,
        [RecordRef::Entity(source.native_id)]
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1,
        "the actual first writer ran"
    );
    failure::assert_model_unchanged(host, &before);
}

#[test]
fn a_foreign_prepared_writer_cannot_publish_the_source_edit() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    refuses_foreign_writer(&world::install());
}
#[test]
fn worker_only_foreign_prepared_writer_cannot_publish_the_source_edit() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    refuses_foreign_writer(&world::install_worker_only());
}

fn replayed_continuation(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
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
    let before = failure::checkpoint(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let _fault = host.runtime().replay_next_coupled_continuation_for_test();
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"replayed-real-continuation".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("a consumed Native continuation must refuse before publication")
    };
    let error = denial
        .authored_retirement_error()
        .expect("Native continuation refusal");
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(AuthoredRetirementRefusal::StaleCoupledContinuation)
    );
    assert_eq!(
        error.context.affected_records,
        [RecordRef::Entity(source.native_id)]
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1
    );
    failure::assert_model_unchanged(&host, &before);
}
#[test]
fn a_real_native_continuation_is_consumed_exactly_once() {
    replayed_continuation(false);
}
#[test]
fn worker_only_real_native_continuation_is_consumed_exactly_once() {
    replayed_continuation(true);
}
