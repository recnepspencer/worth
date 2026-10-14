use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use crate::source_only::{
    fixture,
    readback::{observe_head, read_retention},
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::*,
};

#[test]
fn managed_failure_preserves_unpublished_carrier_and_fresh_recovery() {
    let host = fixture::install();
    let contacts = host.contacts.clone();
    let mut keepalive = (*host).clone();
    assert!(std::ptr::eq(keepalive.runtime(), host.runtime()));
    assert!(std::ptr::eq(
        keepalive.installed_program(),
        host.installed_program()
    ));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let operation = fixture::operation(&host, &request, 18, 0x9176_4018);
    let before = observe_head(host.runtime(), host.current_world())
        .selected_commit()
        .clone();
    let policy = crate::test_request_policy::serial_request();
    host.runtime()
        .fail_next_durable_append_for_test(&worth_execution::ExecutionRequest::serial(&policy));
    let _cleanup = host
        .runtime()
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let operation = fixture::progress(operation, &request);
    super::assert_native_failure(operation.managed_cleanup_failure().unwrap());
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::ProductUnpublished
    );
    assert_eq!(
        observe_head(host.runtime(), host.current_world()).selected_commit(),
        &before
    );
    assert_eq!(contacts.counts(), (1, 1));
    let (operation, denial) = operation
        .close(&request)
        .err()
        .expect("live close does not discharge unpublished custody");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::NativeObligationRetained
    ));
    let stop = WorthQueryCancellationSource::new();
    stop.cancel();
    let stopped_scope = WorthQueryRequestScope::new(None, stop.token());
    let operation = match operation.advance(&host.runtime().request(&principal, &stopped_scope)) {
        WorthQuerySourceOnlyAdvance::Refused { operation, .. } => operation,
        _ => panic!("interrupted admission must return original custody"),
    };
    drop(request);
    drop(principal);
    drop(scope);
    drop(host);
    let denial = keepalive.close_conditional_runtime().unwrap_err();
    assert_eq!(denial.kind(), worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenialKind::RuntimeKeepaliveRetained);
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(keepalive.installed_schema(), &fresh_scope);
    let fresh = keepalive.runtime().request(&fresh_principal, &fresh_scope);
    let operation = fixture::progress(operation, &fresh);
    let completed = fixture::completed(operation, &fresh);
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Recovered { .. }
    ));
    assert_eq!(
        read_retention(keepalive.runtime(), keepalive.current_world()),
        18
    );
    assert_eq!(contacts.counts(), (1, 1));
    let catalog = keepalive
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(catalog.rows().len(), 1);
    let inspector = keepalive
        .runtime()
        .readmit_product_publication_recovery(catalog.rows()[0].handle())
        .unwrap();
    let held = inspector.inspect().unwrap();
    let (completed, denial) = completed
        .close(&fresh)
        .err()
        .expect("a live native inspector refuses release");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::Recovery(_)
    ));
    drop(held);
    completed
        .close(&fresh)
        .unwrap_or_else(|(_, denial)| panic!("fresh cleanup succeeds: {denial:?}"));
    assert!(keepalive
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    drop(fresh);
    drop(inspector);
    keepalive.close_conditional_runtime().unwrap();
}
