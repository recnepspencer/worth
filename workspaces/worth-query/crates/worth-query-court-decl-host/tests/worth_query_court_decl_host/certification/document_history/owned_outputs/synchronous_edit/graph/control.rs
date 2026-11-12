//! Expiry after C principal issuance still vetoes a new applied discovery scope.
use super::*;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionTimeController, WorthQueryCancellationSource, WorthQueryRequestScope,
};
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let controller = WorthQueryAdmissionTimeController::new().unwrap();
    controller.hold();
    let clock = controller.clock();
    let host = world::authorization_clock::install::<program::Program>(clock.clone(), worker, 2);
    let scope =
        WorthQueryRequestScope::new(None, WorthQueryCancellationSource::new().token(), clock);
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::failure::checkpoint(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let control = host
        .runtime()
        .runtime_for_test()
        .primary_graph_for_test()
        .unwrap()
        .retain_entity_resolution_context()
        .advance_before_applied_lookup_for_test(controller, std::time::Duration::from_secs(7200));
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 3,
        })
        .without_source()
        .idempotency(&"expiring-discovery".into())
        .execute_performed::<program::Program, program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        control.was_triggered_for_test(),
        "the real D discovery lookup expires C authentication"
    );
    let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            cause,
        )),
    ) = outcome
    else {
        panic!("expired discovery must refuse the complete edit")
    };
    let denial = cause
        .coupled_producer_denial()
        .expect("true query admission cause");
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::RequestAuthorization(
            WorthQueryOperationAuthorizationRefusalKind::ExpiredAuthentication
        ),
        "{denial:?}"
    );
    assert!(
        denial.subject().contains("owned-outputs.first-binding.v1"),
        "{denial:?}"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1,
        "the parent ran, but no child may run"
    );
    super::super::failure::assert_model_unchanged(&host, &before);
}
#[test]
fn discovery_expiry_after_current_principal_issuance_refuses_without_effect() {
    exercise(false)
}
#[test]
fn worker_only_discovery_expiry_after_current_principal_issuance_refuses_without_effect() {
    exercise(true)
}
