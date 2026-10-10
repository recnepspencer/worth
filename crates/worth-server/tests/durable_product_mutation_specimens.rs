#[path = "support/test_execution_policy.rs"]
mod test_execution_policy;
use serde_json::{json, Value};

#[path = "support/durable_product_mutation/mod.rs"]
mod durable_support;
#[path = "support/route_assembly_phase_twelve/request_driver.rs"]
mod request_driver;

use durable_support::{
    build_server, execute, result_body, session, session_with_principal, TestDurableProductExecutor,
};
use request_driver::WorthServerRouteHttpTestDriver;

struct ProductMutationSpecimen {
    operation_name: &'static str,
    payload_schema: &'static str,
    body: Value,
    expected_schema: &'static str,
    expected_retention: &'static str,
}

#[test]
fn product_shaped_mutations_survive_runtime_reconstruction_once() {
    let executor = TestDurableProductExecutor::default();
    let specimens = product_mutation_specimens();
    let first_server = build_server(&executor);
    let first_session = session(&first_server, "tenant-a", "workspace-42");
    let first = specimens
        .iter()
        .map(|specimen| {
            execute(
                &first_session,
                specimen.operation_name,
                specimen.payload_schema,
                specimen.body.clone(),
                "basis:0",
                "shared-specimen-key",
            )
            .expect("product-shaped durable mutation should commit")
        })
        .collect::<Vec<_>>();
    drop(first_server);

    let rebuilt_server = build_server(&executor);
    let rebuilt_session = session(&rebuilt_server, "tenant-a", "workspace-42");
    for (specimen, original) in specimens.iter().zip(&first) {
        let retried = execute(
            &rebuilt_session,
            specimen.operation_name,
            specimen.payload_schema,
            specimen.body.clone(),
            "basis:0",
            "shared-specimen-key",
        )
        .expect("fresh-runtime retry should resolve the original completion");

        assert!(original.durable_executor_attempted());
        assert!(!original.adapter_execution_attempted());
        assert!(original.retry_diagnostics().is_executed());
        assert!(retried.retry_diagnostics().is_previously_committed());
        assert_eq!(original.result_artifact(), retried.result_artifact());
        let original_receipt = original
            .durable_mutation_receipt()
            .expect("committed durable receipt");
        let retried_receipt = retried
            .durable_mutation_receipt()
            .expect("retried durable receipt");
        assert_eq!(
            original_receipt.completion_digest(),
            retried_receipt.completion_digest()
        );
        assert_eq!(original_receipt.next_basis(), retried_receipt.next_basis());
        assert_eq!(
            original_receipt.product_commit_digest(),
            retried_receipt.product_commit_digest()
        );
        assert_eq!(
            original
                .result_artifact()
                .expect("canonical result artifact")
                .contract()
                .schema()
                .identity(),
            specimen.expected_schema
        );
        assert_eq!(
            result_body(original)["idempotency_retention"],
            specimen.expected_retention
        );
        assert!(original
            .scheduler_admission()
            .expect("durable scheduler admission")
            .scheduler_lane()
            .starts_with("durable-product:"));
    }
    assert_eq!(executor.commit_count(), 3);
}

#[test]
fn durable_attempts_preserve_principal_and_partition_request_identity() {
    let first_executor = TestDurableProductExecutor::default();
    let first_server = build_server(&first_executor);
    execute(
        &session_with_principal(&first_server, "tenant-a", "workspace-42", "principal-alpha"),
        "product.host_connection.upsert",
        "product.host-connection.upsert.v1",
        json!({ "connection_id": "host-7" }),
        "basis:0",
        "principal-bound-key",
    )
    .expect("first principal mutation should commit");

    let second_executor = TestDurableProductExecutor::default();
    let second_server = build_server(&second_executor);
    execute(
        &session_with_principal(&second_server, "tenant-a", "workspace-42", "principal-beta"),
        "product.host_connection.upsert",
        "product.host-connection.upsert.v1",
        json!({ "connection_id": "host-7" }),
        "basis:0",
        "principal-bound-key",
    )
    .expect("second principal mutation should commit in its independent fixture");

    let first = first_executor.observed_attempts();
    let second = second_executor.observed_attempts();
    assert_eq!(first[0].0, "principal-alpha");
    assert_eq!(second[0].0, "principal-beta");
    assert_ne!(first[0].1, second[0].1);
}

#[tokio::test]
async fn declared_http_mutation_projects_result_and_durable_completion_without_a_draft_session() {
    let executor = TestDurableProductExecutor::default();
    let server = build_server(&executor);
    let response = WorthServerRouteHttpTestDriver::new(&server)
        .post_json(
            "/compat/mutations/product.host_connection.upsert?basis=basis:0",
            &[
                ("x-principal-id", "principal-7"),
                ("x-tenant-id", "tenant-a"),
                ("x-workspace-id", "workspace-42"),
                ("x-branch-id", "branch-9"),
                ("idempotency-key", "http-key"),
            ],
            &json!({ "connection_id": "host-http" }),
        )
        .await;

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = response.json_body().expect("JSON response");
    let result = &body["result"];
    assert_eq!(
        result["schema_identity"],
        "product.host-connection.result.v1"
    );
    assert_eq!(result["body"]["resource"]["connection_id"], "host-http");
    assert_eq!(body["durable_completion"]["disposition"], "committed");
    assert_eq!(body["durable_completion"]["next_basis"], "basis:1");
    assert!(body["durable_completion"]["completion_digest"]
        .as_str()
        .is_some_and(|digest| digest.len() == 64));
    assert!(body["durable_completion"]["product_commit_digest"]
        .as_str()
        .is_some());
    assert_eq!(executor.commit_count(), 1);
}

#[tokio::test]
async fn durable_product_rejection_projects_the_product_semantic_denial_class() {
    let executor = TestDurableProductExecutor::default();
    let server = build_server(&executor);
    let response = WorthServerRouteHttpTestDriver::new(&server)
        .post_json(
            "/compat/mutations/product.host_connection.upsert?basis=basis:0",
            &[
                ("x-principal-id", "principal-7"),
                ("x-tenant-id", "tenant-a"),
                ("x-workspace-id", "workspace-42"),
                ("x-branch-id", "branch-9"),
                ("idempotency-key", "http-rejection-key"),
            ],
            &json!({
                "connection_id": "host-http",
                "reject_reason": "host_connection_policy_denied"
            }),
        )
        .await;

    let body = response.json_body().expect("JSON rejection response");
    assert_eq!(body["envelope_kind"], "Denial");
    assert_eq!(
        body["denial"]["reason_key"],
        "host_connection_policy_denied"
    );
    assert_eq!(body["denial"]["code"], "ProductSemantic");
    assert_eq!(
        body["denial"]["detail"],
        "durable product mutation rejected"
    );
    assert!(body["failure"].is_null());
    assert!(body["durable_completion"].is_null());
    assert_eq!(executor.commit_count(), 0);
}

fn product_mutation_specimens() -> [ProductMutationSpecimen; 3] {
    [
        ProductMutationSpecimen {
            operation_name: "product.host_connection.upsert",
            payload_schema: "product.host-connection.upsert.v1",
            body: json!({ "connection_id": "host-7", "endpoint": "https://host.test" }),
            expected_schema: "product.host-connection.result.v1",
            expected_retention: "indefinite",
        },
        ProductMutationSpecimen {
            operation_name: "product.manifest.admit",
            payload_schema: "product.manifest.admit.v1",
            body: json!({ "manifest_digest": "manifest:abc", "admitted": true }),
            expected_schema: "product.manifest.result.v1",
            expected_retention: "at-least-seconds:86400",
        },
        ProductMutationSpecimen {
            operation_name: "product.deployment.transition",
            payload_schema: "product.deployment.transition.v1",
            body: json!({ "deployment_id": "deploy-9", "phase": "active" }),
            expected_schema: "product.deployment.result.v1",
            expected_retention: "at-least-seconds:604800",
        },
    ]
}
