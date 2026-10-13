#[path = "support/test_bootstrap_policy.rs"]
mod test_bootstrap_policy;

use serde_json::json;
use worth_server::{
    WorthServerProductOperationExecutionBoundary, WorthServerProductOperationSurfaceDenialCode,
};

#[path = "support/durable_product_mutation/mod.rs"]
mod durable_support;

use durable_support::{
    build_server, build_server_with_registration, execute, host_registration,
    host_registration_with_support, session, TestDurableProductExecutor,
};

#[test]
fn conflicting_bindings_and_stale_bases_apply_zero_additional_mutations() {
    let executor = TestDurableProductExecutor::default();
    let server = build_server(&executor);
    let operations = session(&server, "tenant-a", "workspace-42");

    let stale = execute(
        &operations,
        "product.deployment.transition",
        "product.deployment.transition.v1",
        json!({ "deployment_id": "deploy-9", "phase": "active" }),
        "basis:99",
        "stale-key",
    )
    .expect_err("stale basis should be denied by product persistence");
    assert_eq!(
        stale.code(),
        WorthServerProductOperationSurfaceDenialCode::PreconditionDenied
    );
    assert_eq!(
        stale.facts().and_then(|facts| facts.execution_boundary()),
        Some(&WorthServerProductOperationExecutionBoundary::DurableExecutorAttempted)
    );
    assert_eq!(executor.commit_count(), 0);

    execute(
        &operations,
        "product.host_connection.upsert",
        "product.host-connection.upsert.v1",
        json!({ "connection_id": "host-7" }),
        "basis:0",
        "conflict-key",
    )
    .expect("first idempotency binding should commit");
    for (body, basis) in [
        (json!({ "connection_id": "host-8" }), "basis:0"),
        (json!({ "connection_id": "host-7" }), "basis:1"),
    ] {
        let conflict = execute(
            &operations,
            "product.host_connection.upsert",
            "product.host-connection.upsert.v1",
            body,
            basis,
            "conflict-key",
        )
        .expect_err("changed payload or basis must conflict with the key binding");
        assert_eq!(
            conflict.code(),
            WorthServerProductOperationSurfaceDenialCode::IdempotencyConflict
        );
    }
    assert_eq!(executor.commit_count(), 1);
}

#[test]
fn operation_declaration_version_is_part_of_the_idempotency_binding() {
    let executor = TestDurableProductExecutor::default();
    let v1_server = build_server_with_registration(host_registration(
        executor.clone(),
        "product.host-connection.upsert.v1",
    ));
    execute(
        &session(&v1_server, "tenant-a", "workspace-42"),
        "product.host_connection.upsert",
        "product.host-connection.upsert.v1",
        json!({ "connection_id": "host-versioned" }),
        "basis:0",
        "version-binding-key",
    )
    .expect("v1 declaration should commit");
    drop(v1_server);

    let v2_server = build_server_with_registration(host_registration(
        executor.clone(),
        "product.host-connection.upsert.v2",
    ));
    let conflict = execute(
        &session(&v2_server, "tenant-a", "workspace-42"),
        "product.host_connection.upsert",
        "product.host-connection.upsert.v2",
        json!({ "connection_id": "host-versioned" }),
        "basis:0",
        "version-binding-key",
    )
    .expect_err("a new declaration version must not inherit the old key binding");
    assert_eq!(
        conflict.code(),
        WorthServerProductOperationSurfaceDenialCode::IdempotencyConflict
    );
    assert_eq!(executor.commit_count(), 1);
}

#[test]
fn declaration_identity_changes_cannot_reuse_an_existing_key_binding() {
    let executor = TestDurableProductExecutor::default();
    let first_server = build_server_with_registration(host_registration_with_support(
        executor.clone(),
        "product.host-connection.upsert.v1",
        "host-upsert-contract-v1",
    ));
    execute(
        &session(&first_server, "tenant-a", "workspace-42"),
        "product.host_connection.upsert",
        "product.host-connection.upsert.v1",
        json!({ "connection_id": "host-declaration-versioned" }),
        "basis:0",
        "declaration-binding-key",
    )
    .expect("first declaration should commit");
    drop(first_server);

    let changed_server = build_server_with_registration(host_registration_with_support(
        executor.clone(),
        "product.host-connection.upsert.v1",
        "host-upsert-contract-v2",
    ));
    let conflict = execute(
        &session(&changed_server, "tenant-a", "workspace-42"),
        "product.host_connection.upsert",
        "product.host-connection.upsert.v1",
        json!({ "connection_id": "host-declaration-versioned" }),
        "basis:0",
        "declaration-binding-key",
    )
    .expect_err("changed declaration identity must conflict with the old key binding");

    assert_eq!(
        conflict.code(),
        WorthServerProductOperationSurfaceDenialCode::IdempotencyConflict
    );
    assert_eq!(executor.commit_count(), 1);
}

#[test]
fn identical_key_text_is_isolated_by_tenant_workspace_and_product_scope() {
    let executor = TestDurableProductExecutor::default();
    let server = build_server(&executor);
    let cases = [
        (
            "tenant-a",
            "workspace-42",
            "product.host_connection.upsert",
            "product.host-connection.upsert.v1",
            json!({ "connection_id": "host-a" }),
        ),
        (
            "tenant-b",
            "workspace-42",
            "product.host_connection.upsert",
            "product.host-connection.upsert.v1",
            json!({ "connection_id": "host-b" }),
        ),
        (
            "tenant-a",
            "workspace-43",
            "product.host_connection.upsert",
            "product.host-connection.upsert.v1",
            json!({ "connection_id": "host-c" }),
        ),
        (
            "tenant-a",
            "workspace-42",
            "product.manifest.admit",
            "product.manifest.admit.v1",
            json!({ "manifest_digest": "manifest:abc" }),
        ),
    ];
    for (tenant, workspace, operation, schema, body) in cases {
        execute(
            &session(&server, tenant, workspace),
            operation,
            schema,
            body,
            "basis:0",
            "same-visible-key",
        )
        .expect("distinct admitted authority scopes must own independent key bindings");
    }
    assert_eq!(executor.commit_count(), 4);
}

#[test]
fn declared_retention_controls_retry_resolution_honestly() {
    let executor = TestDurableProductExecutor::default();
    let first_server = build_server(&executor);
    let body = json!({ "manifest_digest": "manifest:retained" });
    execute(
        &session(&first_server, "tenant-a", "workspace-42"),
        "product.manifest.admit",
        "product.manifest.admit.v1",
        body.clone(),
        "basis:0",
        "retention-key",
    )
    .expect("manifest admission should commit");
    drop(first_server);

    executor.advance_time(86_399);
    let retained_server = build_server(&executor);
    let retained = execute(
        &session(&retained_server, "tenant-a", "workspace-42"),
        "product.manifest.admit",
        "product.manifest.admit.v1",
        body.clone(),
        "basis:0",
        "retention-key",
    )
    .expect("completion must remain resolvable inside declared minimum retention");
    assert!(retained.retry_diagnostics().is_previously_committed());
    drop(retained_server);

    executor.advance_time(1);
    let expired_server = build_server(&executor);
    let expired = execute(
        &session(&expired_server, "tenant-a", "workspace-42"),
        "product.manifest.admit",
        "product.manifest.admit.v1",
        body,
        "basis:0",
        "retention-key",
    )
    .expect_err("after retention expiry the old basis must not silently re-execute");
    assert_eq!(
        expired.code(),
        WorthServerProductOperationSurfaceDenialCode::PreconditionDenied
    );
    assert_eq!(executor.commit_count(), 1);
}
