//! Proposed output relations never grant discovery permission at current C.
use super::*;
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: WorthQueryProgramApplicationRuntime<Schema, program::RestrictedProgram> =
        world::install_program_with_value(limits, 2);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::failure::checkpoint(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 3,
        })
        .without_source()
        .idempotency(&"denied-current-discovery".to_owned())
        .execute_performed::<program::RestrictedProgram, program::RestrictedTree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            cause,
        )),
    ) = outcome
    else {
        panic!("current authority must refuse the complete edit")
    };
    let producer = cause
        .coupled_producer_denial()
        .expect("the original discovery refusal");
    assert_eq!(producer.kind(),WorthQueryOutputDemandDenialKind::SourceScope(worth_query_host::facade::primary_graph::WorthQueryEntityResolutionRefusalKind::UnknownEntity));
    assert!(
        producer
            .subject()
            .contains("owned-outputs.first-discovery-binding.v1"),
        "{producer:?}"
    );
    assert!(
        !producer.subject().contains("EntityId"),
        "the denied public path discloses no proposed identity"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1,
        "the real parent ran before discovery was denied"
    );
    super::super::failure::assert_model_unchanged(&host, &before);
    assert!(request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .is_err());
}
#[test]
fn discovery_cannot_borrow_a_grant_from_its_proposed_parent() {
    exercise(false)
}
#[test]
fn worker_only_discovery_cannot_borrow_a_grant_from_its_proposed_parent() {
    exercise(true)
}
