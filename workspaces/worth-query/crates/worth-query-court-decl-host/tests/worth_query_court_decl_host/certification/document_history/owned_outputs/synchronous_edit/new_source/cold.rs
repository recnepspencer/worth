//! Candidate-created source evidence survives the actual checkpoint boundary.
use super::*;
fn run(worker: bool) {
    use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = if worker {
        world::install_worker_only()
    } else {
        world::install()
    };
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let edit = request
        .advanced()
        .mutate(Intent {
            key: "fresh".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"cold-new-source".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(edit, WorthQueryApplicationMutationOutcome::Committed { .. }),
        "{edit:?}"
    );
    let rows = (
        request
            .query(query::model::Read("fresh".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone(),
        request
            .query(query::first_output::Read("first:fresh".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone(),
        request
            .query(query::second_output::Read("second:fresh".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone(),
        request
            .query(query::child_output::Read("from-first:first:fresh".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone(),
    );
    let checkpoint = super::super::failure::checkpoint(&host);
    drop(request);
    drop(host);
    let cold = world::restore_same_build(checkpoint, worker);
    let head = courts::native_head(&cold);
    let contacts = cold.producer_contacts_on_this_thread_for_test();
    let scope = request_scope();
    let principal = authenticate_operator(cold.installed_schema(), &scope);
    let request = cold.request(&principal, &scope);
    first::settle(&cold, &request, "fresh");
    from_first::settle(&cold, &request, "first:fresh");
    second::settle(&cold, &request, "fresh");
    assert_eq!(
        cold.producer_contacts_on_this_thread_for_test(),
        contacts,
        "restored source facts and witnesses discharge readiness"
    );
    assert_eq!(courts::native_head(&cold), head);
    assert_eq!(
        (
            request
                .query(query::model::Read("fresh".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .clone(),
            request
                .query(query::first_output::Read("first:fresh".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .clone(),
            request
                .query(query::second_output::Read("second:fresh".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .clone(),
            request
                .query(query::child_output::Read("from-first:first:fresh".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .clone(),
        ),
        rows
    );
}
#[test]
fn new_sources_reopen_with_their_original_outputs_and_no_producer_contacts() {
    run(false);
}
#[test]
fn worker_only_new_sources_reopen_with_their_original_outputs_and_no_producer_contacts() {
    run(true);
}
