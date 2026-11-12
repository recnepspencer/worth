//! Discovery support remains live through the final candidate, including own producers.
use super::*;
#[path = "support_program.rs"]
mod support_program;
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host =
        world::install_program_with_other_model::<support_program::Program>(limits, 2, Some(3));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::super::failure::checkpoint(&host);
    let source = request
        .query(query::model::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 7,
        })
        .without_source()
        .idempotency(&"invalidated-root-query".to_owned())
        .execute_performed_discovered::<support_program::Program, support_program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationDiscoveredMutationOutcome::NotPerformed(
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            cause,
        )),
    ) = outcome
    else {
        panic!("the discovered set must remain justified by the final candidate")
    };
    let refusal = cause
        .invariant_execution_failure()
        .expect("the final support owner refuses");
    assert_eq!(
        refusal.kind(),
        WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged
    );
    assert!(
        refusal
            .detail()
            .contains("owned-outputs.member-public-binding.v1")
            && refusal.detail().contains("MemberValue"),
        "{refusal:?}"
    );
    assert_eq!(
        refusal.affected_records(),
        [worth_relational::facade::transactions::RecordRef::Entity(
            source
        )]
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1
    );
    super::super::super::failure::assert_model_unchanged(&host, &before);
    assert!(request
        .query(query::second_output::Read("second:a".into()))
        .execute()
        .is_err());
}
#[test]
fn a_discovered_producer_cannot_invalidate_its_own_root_discovery() {
    exercise(false)
}
#[test]
fn worker_only_a_discovered_producer_cannot_invalidate_its_own_root_discovery() {
    exercise(true)
}
