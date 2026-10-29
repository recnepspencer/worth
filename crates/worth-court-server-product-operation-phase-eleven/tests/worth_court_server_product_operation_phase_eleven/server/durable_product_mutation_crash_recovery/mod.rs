#[allow(unused_imports)] use crate::durable_product_mutation_crash_recovery::ordinary_context::*;
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

pub(crate) use {serde_json::json, serde_json::Value};
pub(crate) use {worth_server::WorthServerCompletedProductOperation, worth_server::WorthServerDurableProductMutationConclusion, worth_server::WorthServerOperationAuthorizationPolicy, worth_server::WorthServerProductOperationOutcome, worth_server::WorthServerProductOperationSurfaceDenialCode};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_durable_product_mutation as durable_support};

pub(crate) use {worth_testkit_server::shared_support_durable_product_mutation::runtime::build_server, worth_testkit_server::shared_support_durable_product_mutation::runtime::build_server_with_mutation_policy, worth_testkit_server::shared_support_durable_product_mutation::driver::execute, worth_testkit_server::shared_support_durable_product_mutation::runtime::prepared_mutation_request, worth_testkit_server::shared_support_durable_product_mutation::runtime::session, worth_testkit_server::shared_support_durable_product_mutation::runtime::session_with_principal, worth_testkit_server::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint, worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor};

#[test]
fn recovery_rechecks_current_operation_authorization() {
    let executor = worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor::default();
    let original_server = build_server(&executor);
    let original_session = session(&original_server, "tenant-a", "workspace-42");
    let denial = execute(
        &original_session,
        "product.deployment.transition",
        "product.deployment.transition.v1",
        crash_payload(worth_testkit_server::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint::AfterCommitBeforeAcknowledgment),
        "basis:0",
        "policy-change-recovery-key",
    )
    .expect_err("post-commit acknowledgment loss should be indeterminate");
    let recovery = denial
        .facts()
        .and_then(|facts| facts.recovery_handle())
        .expect("indeterminate denial should carry recovery authority")
        .clone();
    drop(original_server);

    let rebuilt_server = build_server_with_mutation_policy(
        &executor,
        worth_server::WorthServerOperationAuthorizationPolicy::require_principal("recovery-operator"),
    );
    let no_longer_authorized =
        session_with_principal(&rebuilt_server, "tenant-a", "workspace-42", "principal-7")
            .product_operations()
            .resolve_durable_mutation(&recovery)
            .expect_err("recovery must not bypass the current operation policy");
    assert_eq!(
        no_longer_authorized.code(),
        worth_server::WorthServerProductOperationSurfaceDenialCode::AdmissionDenied
    );
    assert_eq!(
        rebuilt_server.counters().durable_product_recovery_attempts,
        0,
        "authorization denial must occur before product persistence is consulted"
    );

    let authorized_operator = session_with_principal(
        &rebuilt_server,
        "tenant-a",
        "workspace-42",
        "recovery-operator",
    );
    assert!(matches!(
        authorized_operator
            .product_operations()
            .resolve_durable_mutation(&recovery)
            .expect("currently authorized recovery operator should resolve"),
        worth_server::WorthServerDurableProductMutationConclusion::PreviouslyCommitted(_)
    ));
}

#[test]
fn compatibility_recovery_cannot_use_another_operation_route() {
    let executor = worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor::default();
    let server = build_server(&executor);
    let denial = execute(
        &session(&server, "tenant-a", "workspace-42"),
        "product.deployment.transition",
        "product.deployment.transition.v1",
        crash_payload(worth_testkit_server::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint::AfterCommitBeforeAcknowledgment),
        "basis:0",
        "compat-recovery-route-key",
    )
    .expect_err("post-commit acknowledgment loss should be indeterminate");
    let recovery = denial
        .facts()
        .and_then(|facts| facts.recovery_handle())
        .expect("indeterminate denial should carry recovery authority");

    let wrong_route = prepared_mutation_request(&server, "product.host_connection.upsert");
    let denial = server
        .compat_http()
        .product_operations()
        .resolve_durable_mutation(&wrong_route, recovery)
        .expect_err("one compatibility route must not resolve another operation's handle");
    assert_eq!(
        denial.code(),
        worth_server::WorthServerProductOperationSurfaceDenialCode::AdmissionDenied
    );
    assert_eq!(
        server.counters().durable_product_recovery_attempts,
        0,
        "route-binding denial must occur before product persistence is consulted"
    );

    let owning_route = prepared_mutation_request(&server, "product.deployment.transition");
    assert!(matches!(
        server
            .compat_http()
            .product_operations()
            .resolve_durable_mutation(&owning_route, recovery)
            .expect("the admitted owning route should resolve its recovery handle"),
        worth_server::WorthServerDurableProductMutationConclusion::PreviouslyCommitted(_)
    ));
}

#[test]
fn recovery_is_tenant_scoped_and_provider_completion_is_revalidated() {
    let executor = worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor::default();
    let server = build_server(&executor);
    let tenant_a = session(&server, "tenant-a", "workspace-42");
    let denial = execute(
        &tenant_a,
        "product.deployment.transition",
        "product.deployment.transition.v1",
        crash_payload(worth_testkit_server::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint::AfterCommitBeforeAcknowledgment),
        "basis:0",
        "recovery-key",
    )
    .expect_err("post-commit acknowledgment loss should be indeterminate");
    let recovery = denial
        .facts()
        .and_then(|facts| facts.recovery_handle())
        .expect("indeterminate denial should carry recovery authority")
        .clone();

    let cross_tenant = session(&server, "tenant-b", "workspace-42")
        .product_operations()
        .resolve_durable_mutation(&recovery)
        .expect_err("another tenant must not resolve the handle");
    assert_eq!(
        cross_tenant.code(),
        worth_server::WorthServerProductOperationSurfaceDenialCode::RequestDenied
    );

    match tenant_a
        .product_operations()
        .resolve_durable_mutation(&recovery)
        .expect("owning tenant should resolve the handle")
    {
        worth_server::WorthServerDurableProductMutationConclusion::PreviouslyCommitted(completion) => {
            assert_eq!(completion.next_basis().value(), "basis:1");
        }
        other => panic!("expected previously committed recovery, got {other:?}"),
    }

    execute(
        &tenant_a,
        "product.host_connection.upsert",
        "product.host-connection.upsert.v1",
        json!({ "connection_id": "host-7" }),
        "basis:0",
        "unrelated-completion-key",
    )
    .expect("unrelated completion should commit");
    executor.override_recovery_with(executor.completion_for(
        "tenant-a",
        "workspace-42",
        "host-connection",
        "unrelated-completion-key",
    ));
    let invalid = tenant_a
        .product_operations()
        .resolve_durable_mutation(&recovery)
        .expect_err("server must reject a provider completion outside recovery authority");
    assert_eq!(
        invalid.code(),
        worth_server::WorthServerProductOperationSurfaceDenialCode::InvalidDurableCompletion
    );
    assert_eq!(server.counters().durable_product_recovery_failed, 1);
}
