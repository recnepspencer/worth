//! Final qualification retains an ordinary participant's actual decision reads.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
mod program;
fn run(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_value::<program::Program>(limits, 2);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let source = request
        .query(query::model::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    let before = super::failure::checkpoint(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 7,
        })
        .without_source()
        .idempotency(&"ordinary-read-support".to_owned())
        .execute_performed::<program::Program, program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )),
    ) = outcome
    else {
        panic!("a later writer must not invalidate an ordinary participant's decision reads")
    };
    let failure = denial
        .invariant_execution_failure()
        .expect("final candidate support refuses");
    assert_eq!(
        failure.kind(),
        WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged
    );
    assert!(failure.detail().contains("MemberValue"), "{failure:?}");
    assert!(
        failure.detail().contains("later writer producer"),
        "{failure:?}"
    );
    assert_eq!(
        failure.affected_records(),
        [worth_relational::facade::transactions::RecordRef::Entity(
            source
        )]
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test() - contacts,
        2,
        "both real producers ran before final support refusal"
    );
    super::failure::assert_model_unchanged(&host, &before);
    assert!(request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .is_err());
    assert!(request
        .query(query::second_output::Read("second:a".into()))
        .execute()
        .is_err());
}
#[test]
fn ordinary_decision_support_survives_until_final_qualification() {
    run(false)
}
#[test]
fn worker_only_ordinary_decision_support_survives_until_final_qualification() {
    run(true)
}
