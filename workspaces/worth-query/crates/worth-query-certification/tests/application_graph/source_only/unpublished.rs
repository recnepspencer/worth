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

#[test]
fn dropped_ordinary_completion_leaves_independent_native_cleanup() {
    let host = fixture::install();
    let runtime = (*host).clone();
    let contacts = host.contacts.clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let operation = fixture::operation(&host, &request, 18, 0x9176_4218);
    let policy = crate::test_request_policy::serial_request();
    host.runtime()
        .fail_next_durable_append_for_test(&worth_execution::ExecutionRequest::serial(&policy));
    let operation = fixture::progress(operation, &request);
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::ProductUnpublished
    );
    let operation = fixture::progress(operation, &request);
    let completed = fixture::completed(operation, &request);
    assert_eq!(contacts.counts(), (1, 1));
    drop(completed);
    drop(request);
    drop(principal);
    drop(scope);
    drop(host);
    assert_eq!(
        read_retention(runtime.runtime(), runtime.current_world()),
        18
    );
    let catalog = runtime
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(catalog.rows().len(), 1);
    let recovery = runtime
        .readmit_product_publication_recovery(catalog.rows()[0].handle())
        .unwrap();
    let held = recovery.inspect().unwrap();
    let scope = request_scope();
    let failure = runtime
        .with_application_advancement(&scope, |phase| {
            runtime.release_product_publication_recovery_in_advancement(&phase, recovery, 0)
        })
        .unwrap()
        .err()
        .expect("held inspector retains the actual record");
    let recovery = match failure {
        worth_query_host::facade::primary_graph::WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(failure) => failure.into_recovery(),
        _ => panic!("release refusal retains native recovery, not a fabricated cleanup"),
    };
    assert_eq!(
        runtime
            .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
            .unwrap()
            .rows()
            .len(),
        1
    );
    drop(held);
    drop(scope);
    let scope = request_scope();
    let released = runtime
        .with_application_advancement(&scope, |phase| {
            runtime.release_product_publication_recovery_in_advancement(&phase, recovery, 0)
        })
        .unwrap();
    assert!(
        released.is_ok(),
        "fresh issuer-issued native release completes"
    );
    assert!(runtime
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    assert_eq!(contacts.counts(), (1, 1));
}

#[test]
fn explicit_partial_abandonment_keeps_unsettled_native_record() {
    use worth_query_host::facade::primary_graph::{
        RuntimeWorldRecoveryDenial, WorthQueryProductUnpublishedRecoveryReleaseDenial,
        WorthQueryProductUnpublishedRecoveryReleaseFailure,
    };
    let host = fixture::install();
    let contacts = host.contacts.clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let operation = fixture::operation(&host, &request, 18, 0x9176_4318);
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
    let (runtime, _cleanup) = operation.into_runtime_for_cleanup();
    drop(request);
    drop(principal);
    drop(scope);
    drop(host);
    let catalog = runtime
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(catalog.rows().len(), 1);
    let recovery = runtime
        .readmit_product_publication_recovery(catalog.rows()[0].handle())
        .unwrap();
    let scope = request_scope();
    let failure = runtime
        .with_application_advancement(&scope, |phase| {
            runtime.release_product_publication_recovery_in_advancement(&phase, recovery, 0)
        })
        .unwrap()
        .err()
        .expect("unsettled source effects are not implicitly disposed");
    let recovery = match failure {
        WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(failure) => {
            assert_eq!(
                failure.denial(),
                WorthQueryProductUnpublishedRecoveryReleaseDenial::World(
                    RuntimeWorldRecoveryDenial::SettlementRequired
                )
            );
            failure.into_recovery()
        }
        _ => panic!("unsettled native record must remain actual recovery custody"),
    };
    assert_eq!(recovery.record_handle(), catalog.rows()[0].handle());
    assert_eq!(
        runtime
            .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
            .unwrap()
            .rows()
            .len(),
        1
    );
    assert_eq!(
        observe_head(runtime.runtime(), runtime.current_world()).selected_commit(),
        &before
    );
    assert_eq!(contacts.counts(), (1, 1));
    // No invented settlement/close: the native record remains independently discoverable.
    drop(recovery);
}
