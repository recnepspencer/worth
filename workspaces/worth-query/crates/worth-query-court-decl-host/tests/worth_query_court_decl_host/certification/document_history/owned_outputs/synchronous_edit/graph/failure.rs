//! A discovered child's refusal discards its source and actual parent writer.
use super::*;
fn reject_child(value: u64) -> u64 {
    if value == 3 {
        u64::MAX
    } else {
        second::derived(value)
    }
}
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_configured::<program::Program>(
        limits,
        2,
        None,
        world::Configuration {
            reconciliation_indices: false,
            reconciliation_peer_caps: false,
            reconciliation_emission: false,
            envelope_records: false,
            second_derivation: Some(reject_child),
        },
    );
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
        .idempotency(&"refused-discovered-child".to_owned())
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
        panic!("the complete edit must refuse")
    };
    let producer = cause
        .coupled_producer_denial()
        .expect("the true child's typed refusal");
    assert_eq!(
        producer.kind(),
        WorthQueryOutputDemandDenialKind::ProducerDomainDenied
    );
    assert!(
        producer.subject().contains("owned-outputs.second.initial"),
        "{producer:?}"
    );
    assert_eq!(producer.domain_reason(), Some("DerivedValueOutOfRange"));
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "the real parent and discovered child both ran"
    );
    super::super::failure::assert_model_unchanged(&host, &before);
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
fn a_discovered_child_refusal_leaves_the_whole_model_byte_identical() {
    exercise(false)
}
#[test]
fn worker_only_discovered_child_refusal_leaves_the_whole_model_byte_identical() {
    exercise(true)
}
