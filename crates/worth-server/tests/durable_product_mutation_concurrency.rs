#[path = "support/test_execution_policy.rs"]
mod test_execution_policy;
use serde_json::json;
use worth_server::WorthServerProductOperationSurfaceDenialCode;

#[path = "support/durable_product_mutation/mod.rs"]
mod durable_support;

use durable_support::{
    build_server, execute, session, TestConcurrencyProbe, TestDurableProductExecutor,
};

#[test]
fn disjoint_product_scopes_enter_transactions_concurrently() {
    let probe = TestConcurrencyProbe::expecting(2);
    let executor = TestDurableProductExecutor::with_concurrency_probe(probe.clone());
    let server = build_server(&executor);
    let session = session(&server, "tenant-a", "workspace-42");
    let operations = [
        (
            "product.manifest.admit",
            "product.manifest.admit.v1",
            json!({ "manifest_digest": "manifest:abc", "concurrency_probe": true }),
            "manifest-concurrency-key",
        ),
        (
            "product.deployment.transition",
            "product.deployment.transition.v1",
            json!({
                "deployment_id": "deploy-9",
                "phase": "active",
                "concurrency_probe": true,
            }),
            "deployment-concurrency-key",
        ),
    ];
    let outcomes = std::thread::scope(|scope| {
        operations
            .into_iter()
            .map(|(operation, schema, body, key)| {
                let session = session.clone();
                scope.spawn(move || execute(&session, operation, schema, body, "basis:0", key))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|thread| {
                thread
                    .join()
                    .expect("durable concurrency thread should join")
            })
            .collect::<Vec<_>>()
    });

    assert!(outcomes.iter().all(Result::is_ok));
    assert_eq!(probe.maximum_active_entries(), 2);
    assert_eq!(executor.commit_count(), 2);
}

#[test]
fn same_scope_same_basis_chooses_one_winner() {
    let executor = TestDurableProductExecutor::default();
    let first_server = build_server(&executor);
    let second_server = build_server(&executor);
    let sessions = [
        session(&first_server, "tenant-a", "workspace-42"),
        session(&second_server, "tenant-a", "workspace-42"),
    ];
    let outcomes = std::thread::scope(|scope| {
        [
            (
                sessions[0].clone(),
                json!({ "connection_id": "host-7" }),
                "same-scope-a",
            ),
            (
                sessions[1].clone(),
                json!({ "connection_id": "host-8" }),
                "same-scope-b",
            ),
        ]
        .into_iter()
        .map(|(session, body, key)| {
            scope.spawn(move || {
                execute(
                    &session,
                    "product.host_connection.upsert",
                    "product.host-connection.upsert.v1",
                    body,
                    "basis:0",
                    key,
                )
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|thread| thread.join().expect("same-scope thread should join"))
        .collect::<Vec<_>>()
    });

    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| outcome.as_ref().is_err_and(|denial| {
                denial.code() == WorthServerProductOperationSurfaceDenialCode::PreconditionDenied
            }))
            .count(),
        1
    );
    assert_eq!(executor.commit_count(), 1);
}

#[test]
fn durable_runtime_counters_report_exact_conclusions_and_result_bytes() {
    let executor = TestDurableProductExecutor::default();
    let server = build_server(&executor);
    let session = session(&server, "tenant-a", "workspace-42");
    let committed = execute(
        &session,
        "product.host_connection.upsert",
        "product.host-connection.upsert.v1",
        json!({ "connection_id": "host-7" }),
        "basis:0",
        "counter-key",
    )
    .expect("first counter specimen should commit");
    execute(
        &session,
        "product.host_connection.upsert",
        "product.host-connection.upsert.v1",
        json!({ "connection_id": "host-7" }),
        "basis:0",
        "counter-key",
    )
    .expect("identical retry should resolve");
    execute(
        &session,
        "product.host_connection.upsert",
        "product.host-connection.upsert.v1",
        json!({ "connection_id": "host-8" }),
        "basis:0",
        "counter-key",
    )
    .expect_err("changed retry should conflict");
    execute(
        &session,
        "product.deployment.transition",
        "product.deployment.transition.v1",
        json!({ "deployment_id": "deploy-9", "phase": "active" }),
        "basis:99",
        "stale-counter-key",
    )
    .expect_err("stale counter specimen should deny");
    execute(
        &session,
        "product.manifest.admit",
        "product.manifest.admit.v1",
        json!({ "oversized_result": true }),
        "basis:0",
        "oversized-counter-key",
    )
    .expect_err("oversized result should deny");

    let counters = server.counters();
    assert_eq!(counters.durable_product_mutation_attempts, 5);
    assert_eq!(counters.durable_product_basis_comparisons, 3);
    assert_eq!(counters.durable_product_commits, 1);
    assert_eq!(counters.durable_product_previously_committed, 1);
    assert_eq!(counters.durable_product_idempotency_conflicts, 1);
    assert_eq!(counters.durable_product_stale_bases, 1);
    assert_eq!(counters.product_result_oversized_denials, 1);
    assert_eq!(counters.product_result_artifacts_emitted, 2);
    assert_eq!(
        counters.product_result_bytes_emitted,
        (committed
            .result_artifact()
            .expect("committed artifact")
            .body()
            .byte_len()
            * 2) as u64,
    );
}
