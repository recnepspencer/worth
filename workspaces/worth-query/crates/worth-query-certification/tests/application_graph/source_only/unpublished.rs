use super::{
    fixture,
    readback::{observe_head, read_retention},
};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::*,
};

#[test]
fn owning_source_only_unpublished_recovery_and_close() {
    let host = fixture::install();
    let contacts = &host.contacts;
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
    let operation = fixture::progress(operation, &request);
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
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
    let fresh = host.runtime().request(&fresh_principal, &fresh_scope);
    let operation = fixture::progress(operation, &fresh);
    let completed = fixture::completed(operation, &fresh);
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Recovered { .. }
    ));
    assert_eq!(read_retention(host.runtime(), host.current_world()), 18);
    assert_eq!(contacts.counts(), (1, 1));
    let catalog = host
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(catalog.rows().len(), 1);
    let inspector = host
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
    assert!(host
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
}

#[test]
fn original_nondefault_precondition_refuses_without_source_effect() {
    let host = fixture::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let before = observe_head(host.runtime(), host.current_world())
        .selected_commit()
        .clone();
    let operation =
        fixture::operation_with_expected(&host, &request, 18, 0x9176_4118, "wrong-original-fact");
    match operation.advance(&request) {
        WorthQuerySourceOnlyAdvance::Completed(_) => panic!("the captured condition must refuse"),
        WorthQuerySourceOnlyAdvance::Terminal(terminal) => {
            assert!(matches!(terminal.native_outcome(), Some(WorthQueryApplicationMutationOutcome::Commit(worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted::Denied(_)))), "retain the real native precondition refusal");
            terminal.close(&request).unwrap_or_else(|(_, denial)| {
                panic!("the native no-effect terminal closes: {denial:?}")
            });
        }
        WorthQuerySourceOnlyAdvance::Refused { .. } => {}
        WorthQuerySourceOnlyAdvance::Progress(_) => {
            panic!("a wrong fact has no pending publication")
        }
    }
    assert_eq!(
        observe_head(host.runtime(), host.current_world()).selected_commit(),
        &before
    );
    assert_eq!(read_retention(host.runtime(), host.current_world()), 7);
    assert_eq!(host.contacts.counts(), (1, 0));
}
