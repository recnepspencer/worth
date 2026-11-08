//! Discovery support changed by a later child must refuse before publication.
use super::*;
pub(super) mod program;
pub mod query;
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: WorthQueryProgramApplicationRuntime<Schema, program::Program> =
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
        .idempotency(&"unstable-discovery".to_owned())
        .execute_performed::<program::Program, program::Tree>(
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
        panic!("changed discovery support must refuse the whole edit")
    };
    let failure = cause
        .invariant_execution_failure()
        .expect("the existing candidate-support owner refuses");
    assert_eq!(failure.kind(),worth_query_host::facade::primary_graph::WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged);
    assert!(
        failure.detail().contains("discovery support")
            && failure
                .detail()
                .contains("owned-outputs.unstable-discovery-binding.v1"),
        "{failure:?}"
    );
    let source = request
        .query(super::super::super::query::model::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    assert_eq!(
        failure.affected_records(),
        [worth_relational::facade::transactions::RecordRef::Entity(
            source
        )]
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
    super::super::failure::assert_model_unchanged(&host, &before);
}
#[test]
fn a_child_that_changes_discovery_support_refuses_the_whole_edit() {
    exercise(false)
}
#[test]
fn worker_only_child_that_changes_discovery_support_refuses_the_whole_edit() {
    exercise(true)
}
