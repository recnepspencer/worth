//! Refusing a descendant discards the authored prefix and all prepared roots.
use super::*;
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
        Some(3),
        world::Configuration {
            reconciliation_indices: false,
            reconciliation_peer_caps: false,
            reconciliation_emission: false,
            second_derivation: Some(|_| u64::MAX),
        },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::super::failure::checkpoint(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 7,
        })
        .without_source()
        .idempotency(&"failed-discovered-root-child".to_owned())
        .execute_performed_discovered::<program::Program, program::Tree>(
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
        panic!("the failed descendant must refuse the complete edit")
    };
    let refusal = cause
        .coupled_producer_denial()
        .expect("the real descendant refusal");
    assert_eq!(
        refusal.kind(),
        WorthQueryOutputDemandDenialKind::ProducerDomainDenied
    );
    assert!(
        refusal.subject().contains("owned-outputs.second.initial"),
        "{refusal:?}"
    );
    assert_eq!(refusal.domain_reason(), Some("DerivedValueOutOfRange"));
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 3,
        "both parents prepared before the refused child"
    );
    super::super::super::failure::assert_model_unchanged(&host, &before);
    assert!(request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .is_err());
    assert!(request
        .query(query::first_output::Read("first:b".into()))
        .execute()
        .is_err());
}
#[test]
fn descendant_refusal_leaves_all_discovered_roots_and_the_source_byte_identical() {
    exercise(false)
}
#[test]
fn worker_only_descendant_refusal_leaves_all_discovered_roots_and_the_source_byte_identical() {
    exercise(true)
}
